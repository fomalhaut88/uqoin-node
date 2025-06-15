use tokio::io::Result as TkResult;
use actix_web::{web, HttpResponse, Result as ActixResult};

use crate::error::JsonError;
use crate::appdata::AppData;


pub type TokioResult<T> = TkResult<T>;
pub type APIResult = ActixResult<HttpResponse, JsonError>;
pub type WebAppData = web::Data<AppData>;


/// This function searchs for `ix` such that `check(ix) == true` and 
/// `check(ix+1) == false`. `check` must satisty `check(ix1) >= check(ix2)` for
/// `ix1 < ix2`. `ix` should be from `0` to `ix_last` inclusively. The
/// algorithm is optimized for the result close to `ix_last`. If 
/// `check(0) == false` the function returns `None` and if 
/// `check(ix_last) == true` the result is `ix_last`.
pub async fn find_divergence<F>(ix_last: u64, check: F) -> TkResult<Option<u64>> 
                                where F: AsyncFn(u64) -> TkResult<bool> {
    if ix_last == 0 {
        // Case `ix_last = 0`
        if check(0).await? {
            Ok(Some(0))
        } else {
            Ok(None)
        }
    } else if check(ix_last).await? {
        // Case `ix_last` gives `true`
        Ok(Some(ix_last))
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

        // Check 0 and return the result
        if ix_from == 0 && !check(0).await? {
            Ok(None)
        } else {
            Ok(Some(ix_from))
        }
    }
}


/// Run async task up to `count` times to get the result or error if all failed.
#[macro_export]
macro_rules! async_try_many {
    ($count:expr, $func:ident $(, $arg:expr)*) => {
        {
            let mut res = Err(ErrorKind::Other.into());
            for _ in 0..$count {
                res = $func($($arg,)*).await;
                if res.is_ok() {
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
        assert_eq!(find_divergence(10, async |ix| Ok(ix <= 7)).await?, Some(7));
        assert_eq!(find_divergence(10, async |ix| Ok(ix <= 1)).await?, Some(1));
        assert_eq!(find_divergence(10, async |ix| Ok(ix <= 9)).await?, Some(9));
        assert_eq!(find_divergence(10, async |ix| Ok(ix <= 5)).await?, Some(5));
        assert_eq!(find_divergence(10, async |ix| Ok(ix <= 0)).await?, Some(0));
        assert_eq!(find_divergence(10, async |_| Ok(false)).await?, None);
        assert_eq!(find_divergence(10, async |_| Ok(true)).await?, Some(10));
        assert_eq!(find_divergence(0, async |_| Ok(false)).await?, None);
        assert_eq!(find_divergence(0, async |_| Ok(true)).await?, Some(0));
        Ok(())
    }
}
