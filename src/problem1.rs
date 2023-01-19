use crate::prelude::*;

use tokio::sync::RwLock;

mod parse {
    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::character::complete::{char, multispace0 as space};
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

    fn method(input: &[u8]) -> IResult<&[u8], ()> {
        let key = tag(r#""method""#);
        let value = tag(r#""isPrime""#);

        let sep = tuple((space, keysep, space));

        let (input, _) = seppair(key, sep, value)(input)?;
        Ok((input, ()))
    }

    fn number(input: &[u8]) -> IResult<&[u8], u32> {
        let key = tag(r#""number""#);
        preceded(tuple((key, keysep)), nom::character::complete::u32)(input)
    }

    pub fn all(input: &[u8]) -> IResult<&[u8], u32> {
        let method_number = preceded(tuple((method, itemsep)), number);
        let number_method = terminated(number, tuple((itemsep, method)));
        delimited(
            tuple((char('{'), space)),
            alt((method_number, number_method)),
            tuple((space, char('}'))),
        )(input)
    }
}

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("I/O error: {0:?}")]
    IOError(#[from] io::Error),
    #[error("prime logic error: {0:?}")]
    SieveError(#[from] SieveError),
}

#[derive(ThisError, Debug)]
pub enum SieveError {
    #[error("{0} is not contained in this sieve of len {1}")]
    OutOfBounds(usize, usize),
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

    fn expand(&mut self, new_limit: usize) {
        if self.contains(new_limit) {
            return;
        }
        let new_limit = new_limit.clamp(Self::MIN_CAPACITY, usize::MAX);

        for _ in 0..(new_limit - self.0.len()) {
            self.0.push(true);
        }

        for mult in 2..self.0.len() {
            let mut n = mult * 2;
            while n <= self.0.len() {
                self.0[n - 1] = false;
                n += mult;
            }
        }
    }

    fn get(&self, num: usize) -> Result<bool, SieveError> {
        if !self.contains(num) {
            return Err(SieveError::OutOfBounds(num, self.0.len()));
        }

        Ok(self.0[num - 1])
    }
}

#[derive(Debug)]
pub struct PrimeTime {
    sieve: RwLock<PrimeSieve>,
}

impl PrimeTime {
    pub fn new() -> Self {
        Self {
            sieve: RwLock::new(PrimeSieve::with_capacity(1000)),
        }
    }

    fn form_resp(value: bool) -> Vec<u8> {
        format!("{{\"method\":\"isPrime\",\"prime\":{value}}}\n").into_bytes()
    }
}

impl Server<Error> for PrimeTime {}

impl Service<Vec<u8>> for PrimeTime {
    type Error = Error;
    type Response = Vec<u8>;
    type Future = Pin<Box<dyn Future<Output = Result<Vec<u8>, Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Error>> {
        match self.sieve.try_read() {
            Ok(_) => Poll::Ready(Ok(())),
            Err(_) => Poll::Pending,
        }
    }

    fn call(&mut self, req: Vec<u8>) -> Self::Future {
        let num = if let Ok((_, n)) = parse::all(&req) {
            n as usize
        } else {
            trace!("malformed request: {}", String::from_utf8_lossy(&req));
            return Box::pin(async move { Ok("{}\n".to_string().into_bytes()) });
        };

        {
            let sieve = match self.sieve.try_read() {
                Ok(s) => s,
                Err(_) => todo!("yield execution"),
            };

            if let Ok(p) = sieve.get(num) {
                trace!("number {num} is prime = {p}");
                return Box::pin(async move { Ok(Self::form_resp(p)) });
            }
            // otherwise fall through to expanding the sieve
        }

        let mut sieve = match self.sieve.try_write() {
            Ok(s) => s,
            Err(_) => todo!("yield execution"),
        };

        debug!("expanding prime sieve to value {num}");
        sieve.expand(num);
        let is_prime = sieve.get(num).unwrap();
        Box::pin(async move { Ok(Self::form_resp(is_prime)) })
    }
}
