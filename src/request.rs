use std::collections::HashMap;

use nom::{
    bytes::complete::{tag, take_until, take_while1},
    multi::fold_many0,
    IResult,
};

#[derive(Debug)]
pub struct RequestLine<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub version: &'a str,
}

impl<'a> RequestLine<'a> {
    pub fn parse(input: &'a str) -> IResult<&str, Self> {
        let (input, method) = take_while1(|c| c != ' ')(input)?;
        let (input, _) = tag(" ")(input)?;
        let (input, path) = take_until(" ")(input)?;
        let (input, _) = tag(" ")(input)?;
        let (input, version) = take_until("\r\n")(input)?;
        let (input, _) = tag("\r\n")(input)?;

        Ok((
            input,
            Self {
                method,
                path,
                version,
            },
        ))
    }
}

#[derive(Debug)]
pub struct Headers<'a>(pub HashMap<&'a str, &'a str>);

impl<'a> Headers<'a> {
    pub fn parse(input: &'a str) -> IResult<&str, Self> {
        let (input, headers) = fold_many0(
            Self::parse_header,
            || HashMap::new(),
            |mut map, (key, value)| {
                map.insert(key, value);
                map
            },
        )(input)?;
        Ok((input, Self(headers)))
    }

    fn parse_header(input: &'a str) -> IResult<&str, (&str, &str)> {
        let (input, key) = take_until(": ")(input)?;
        let (input, _) = tag(": ")(input)?;
        let (input, value) = take_until("\r\n")(input)?;
        let (input, _) = tag("\r\n")(input)?;

        Ok((input, (key, value)))
    }
}

#[derive(Debug)]
pub struct Body(pub Vec<u8>);

impl Body {
    pub fn parse(input: &str) -> IResult<&str, Option<Self>> {
        let (input, data) = take_until("\0")(input)?;
        if data.is_empty() {
            Ok((input, None))
        } else {
            Ok((input, Some(Self(data.as_bytes().to_vec()))))
        }
    }
}

#[derive(Debug)]
pub struct Request<'a> {
    pub request_line: RequestLine<'a>,
    pub headers: Headers<'a>,
    pub body: Option<Body>,
}

impl<'a> Request<'a> {
    pub fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (input, request_line) = RequestLine::parse(input)?;
        let (input, headers) = Headers::parse(input)?;
        let (_, body) = Body::parse(input)?;
        Ok((
            input,
            Self {
                request_line,
                headers,
                body,
            },
        ))
    }

    pub fn path(&self) -> &str {
        self.request_line.path
    }

    pub fn method(&self) -> &str {
        self.request_line.method
    }
}
