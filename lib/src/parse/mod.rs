use bytes::Buf;

pub trait Parser<'a, STREAM: 'a, T, E> {
    fn parse(&self, buf: &'a [STREAM]) -> Result<(T, &'a [STREAM]), E>;
}

pub trait ParserExt<'a, STREAM: 'a, T, E>: Parser<'a, STREAM, T, E> + Sized {
    #[inline]
    fn map<U>(self, f: impl Fn(T) -> U) -> impl Parser<'a, STREAM, U, E> {
        move |buf| -> Result<(U, &[STREAM]), E> {
            let (res, buf) = self.parse(buf)?;
            Ok((f(res), buf))
        }
    }

    #[inline]
    fn flat_map<U, P: Parser<'a, STREAM, U, E>>(self, f: impl Fn(T) -> P) -> impl Parser<'a, STREAM, U, E> {
        move |buf| -> Result<(U, &'a [STREAM]), E> {
            let (res, buf) = self.parse(buf)?;
            f(res).parse(buf)
        }
    }

    #[inline]
    fn optional(self) -> impl Parser<'a, STREAM, Option<T>, E> {
        move |buf| -> Result<(Option<T>, &'a [STREAM]), E> {
            match self.parse(buf) {
                Ok((res, buf)) => Ok((Some(res), buf)),
                Err(_) => Ok((None, buf)),
            }
        }
    }

    #[inline]
    fn then<U>(self, then: impl Parser<'a, STREAM, U, E>) -> impl Parser<'a, STREAM, (T, U), E> {
        move |buf| -> Result<((T, U), &'a [STREAM]), E> {
            let (l, buf) = self.parse(buf)?;
            let (r, buf) = then.parse(buf)?;
            Ok(((l, r), buf))
        }
    }

    #[inline]
    fn then_right<U>(self, then: impl Parser<'a, STREAM, U, E>) -> impl Parser<'a, STREAM, U, E> {
        self.then(then).map(|(_, r)| r)
    }

    #[inline]
    fn then_left<U>(self, then: impl Parser<'a, STREAM, U, E>) -> impl Parser<'a, STREAM, T, E> {
        self.then(then).map(|(l, _)| l)
    }
}

impl <'a, STREAM: 'a, T, E, P: Parser<'a, STREAM, T, E> + Sized> ParserExt<'a, STREAM, T, E> for P {}

impl <'a, STREAM: 'a, T, E, F: Fn(&'a [STREAM]) -> Result<(T, &'a [STREAM]), E>> Parser<'a, STREAM, T, E> for F {
    fn parse(&self, buf: &'a [STREAM]) -> Result<(T, &'a [STREAM]), E> {
        self(buf)
    }
}

fn match_buf<'a, STREAM>(match_buf: &'a [STREAM]) -> impl Parser<'a, STREAM, &'a [STREAM], ()>
where
    STREAM: Eq
{
    move |buf: &'a [STREAM]| -> Result<(&'a [STREAM], &'a [STREAM]), ()> {
        let matches = match_buf.iter().zip(buf.iter()).all(|(l, r)| l == r);
        if matches {
            Ok((&buf[..match_buf.len()], &buf[match_buf.len()..]))
        }
        else {
            Err(())
        }
    }
}


