use std::sync::Arc;

use crate::prelude::*;

use tokio::sync::RwLock;

mod parse {
    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::character::complete::{char, line_ending, multispace0 as space};
    use nom::sequence::{delimited, preceded, separated_pair as seppair, terminated, tuple};
    use nom::IResult;

    fn keysep(input: &[u8]) -> IResult<&[u8], ()> {
        let (input, _) = tuple((space, char(':'), space))(input)?;
        Ok((input, ()))
    }

    fn itemsep(input: &[u8]) -> IResult<&[u8], ()> {
        let (input, _) = tuple((space, char(','), space))(input)?;
        Ok((input, ()))
    }

    fn quoted(key: &[u8]) -> impl Fn(&[u8]) -> IResult<&[u8], ()> + '_ {
        move |input| {
            let (input, _) = alt((
                delimited(char('"'), tag(key), char('"')),
                delimited(char('\''), tag(key), char('\'')),
            ))(input)?;
            Ok((input, ()))
        }
    }

    fn method(input: &[u8]) -> IResult<&[u8], ()> {
        let (input, _) = seppair(quoted(b"method"), keysep, quoted(b"isPrime"))(input)?;
        Ok((input, ()))
    }

    fn number(input: &[u8]) -> IResult<&[u8], u32> {
        preceded(
            tuple((quoted(b"number"), keysep)),
            nom::character::complete::u32,
        )(input)
    }

    pub fn all(input: &[u8]) -> IResult<&[u8], u32> {
        let method_number = preceded(tuple((method, itemsep)), number);
        let number_method = terminated(number, tuple((itemsep, method)));
        delimited(
            tuple((char('{'), space)),
            alt((method_number, number_method)),
            tuple((space, char('}'), line_ending)),
        )(input)
    }
}

#[derive(Debug)]
struct PrimeSieve(Vec<bool>);

impl PrimeSieve {
    const MIN_CAPACITY: usize = 10;

    fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    fn contains(&self, target: usize) -> bool {
        target - 1 < self.0.len()
    }

    fn expand(&mut self, new_limit: usize) -> bool {
        if self.contains(new_limit) {
            return self.0[new_limit - 1];
        }
        let new_limit = new_limit.clamp(Self::MIN_CAPACITY, usize::MAX);

        self.0.resize(new_limit, true);

        for mult in 2..self.0.len() {
            // TODO: we can skip quite a lot of assignments by starting
            // higher than this
            let mut n = mult * 2;
            while n <= self.0.len() {
                self.0[n - 1] = false;
                n += mult;
            }
        }

        self.0[new_limit - 1]
    }

    /// Try to get a prime from the sieve
    ///
    /// If the sieve doesn't contain the number yet, returns `None`.
    fn get(&self, num: usize) -> Option<bool> {
        if self.contains(num) {
            Some(self.0[num - 1])
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct PrimeTime {
    sieve: Arc<RwLock<PrimeSieve>>,
}

impl Default for PrimeTime {
    fn default() -> Self {
        Self {
            sieve: Arc::new(RwLock::new(PrimeSieve::with_capacity(1000))),
        }
    }
}

impl PrimeTime {
    #[instrument]
    fn form_resp(value: bool) -> Vec<u8> {
        let mut resp = format!(r#"{{"method":"isPrime","prime":{value}}}"#);
        resp.push('\n');
        debug!("responding: {resp}");
        resp.into_bytes()
    }
}

impl Service<Vec<u8>> for PrimeTime {
    type Error = Infallible;
    type Response = Vec<u8>;
    type Future = Pin<Box<dyn Future<Output = Result<Vec<u8>, Infallible>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Infallible>> {
        match self.sieve.try_read() {
            Ok(_) => Poll::Ready(Ok(())),
            Err(_) => Poll::Pending,
        }
    }

    fn call(&mut self, req: Vec<u8>) -> Self::Future {
        let num = if let Ok((_, n)) = parse::all(&req) {
            n as usize
        } else {
            if req.len() < 30 {
                let prev = String::from_utf8_lossy(&req);
                trace!("malformed request: {prev}");
            } else {
                trace!("malformed request, too long to display");
            }
            return Box::pin(async { Ok("{}\n".to_string().into_bytes()) });
        };
        debug!("request received for {num}");

        let sieve_handle = Arc::clone(&self.sieve);
        Box::pin(async move {
            let res = { sieve_handle.read().await.get(num) };

            let is_prime = if let Some(p) = res {
                p
            } else {
                // otherwise fall through to expanding the sieve
                let mut sieve = sieve_handle.write_owned().await;
                debug!("expanding prime sieve to value {num}");
                tokio::task::spawn_blocking(move || sieve.expand(num))
                    .await
                    .unwrap()
            };

            trace!("number {num} is prime: {is_prime}");
            Ok(Self::form_resp(is_prime))
        })
    }
}

impl Server for PrimeTime {}

#[cfg(test)]
mod tests {
    use super::PrimeTime;

    #[test]
    fn valid_resp_form() {
        use serde_json::Value;

        let resp_raw = PrimeTime::form_resp(false);
        let resp: Value = serde_json::from_slice(&resp_raw).unwrap();

        let expected = Value::Object({
            let mut map = serde_json::Map::new();
            map.insert("method".to_owned(), Value::String("isPrime".to_owned()));
            map.insert("prime".to_owned(), Value::Bool(false));
            map
        });
        assert_eq!(expected, resp);
    }
}
