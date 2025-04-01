use tokio::io::Result as TkResult;
use serde::Serialize;
use actix_web::{web, HttpResponse, Result as ActixResult, Error as ActixError};
use uqoin_core::error::{Error as UqoinError};

use crate::appdata::AppData;


pub type TokioResult<T> = TkResult<T>;
pub type APIResult = ActixResult<HttpResponse, ActixError>;
pub type WebAppData = web::Data<AppData>;


/// Error structure that is convertable from uqoin-core errors.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    error: String,
}


impl ErrorResponse {
    /// Create a new ErrorResponse instance.
    pub fn new(message: &str) -> Self {
        Self { error: message.to_string() }
    }
}


impl From<UqoinError> for ErrorResponse {
    fn from(err: UqoinError) -> Self {
        Self::new(&err.to_string())
    }
}


/// This function searchs for `ix` such that `check(ix) == true` and 
/// `check(ix+1) == false`. `check` must satisty `check(ix1) >= check(ix2)` for
/// `ix1 < ix2`. `ix` should be from `1` to `ix_last` inclusively. The
/// algorithm is optimized for the result close to `ix_last`. If 
/// `check(1) == false` the function returns `0` and if `check(ix_last) == true`
/// the result is `ix_last`.
pub async fn find_divergence<F>(ix_last: u64, check: F) -> TkResult<u64> 
                                where F: AsyncFn(u64) -> TkResult<bool> {
    // Check `ix_last`
    if ix_last == 0 || check(ix_last).await? {
        Ok(ix_last)
    } else {
        // Going down with exponential step
        let mut step = 1;
        let mut ix_to = ix_last;
        let mut ix_from = ix_last - step;
        
        while !check(ix_from).await? {
            ix_to = ix_from;
            step <<= 1;
            if ix_from > step {
                ix_from -= step;
            } else {
                ix_from = 0;
                break;
            }
        }

        // Binary search in `[ix_from, ix_to)`
        while ix_to - ix_from > 1 {
            let ix_mid = (ix_to + ix_from) >> 1;
            if !check(ix_mid).await? {
                ix_to = ix_mid;
            } else {
                ix_from = ix_mid;
            }
        }

        Ok(ix_from)
    }
}


/// Run async task up to `count` times to get the result or error if all failed.
#[macro_export]
macro_rules! async_try_many {
    ($count:expr, $func:ident $(, $arg:expr)*) => {
        {
            let mut res = Err(Error::new(ErrorKind::Other, "Too many errors"));
            for _ in 0..$count {
                if let Ok(r) = $func($($arg,)*).await {
                    res = Ok(r);
                    break;
                }
            }
            res
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_divergence() -> TkResult<()> {
        assert_eq!(find_divergence(10, async |ix| Ok(ix <= 7)).await?, 7);
        assert_eq!(find_divergence(10, async |ix| Ok(ix <= 1)).await?, 1);
        assert_eq!(find_divergence(10, async |ix| Ok(ix <= 9)).await?, 9);
        assert_eq!(find_divergence(10, async |ix| Ok(ix <= 5)).await?, 5);
        assert_eq!(find_divergence(10, async |_| Ok(false)).await?, 0);
        assert_eq!(find_divergence(10, async |_| Ok(true)).await?, 10);
        assert_eq!(find_divergence(0, async |_| Ok(false)).await?, 0);
        assert_eq!(find_divergence(0, async |_| Ok(true)).await?, 0);
        Ok(())
    }
}
