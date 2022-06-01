use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take};
use nom::character::complete::alpha1;
use nom::error::{context, ErrorKind, VerboseError};
use nom::multi::{many1, many_m_n};
use nom::sequence::{terminated, tuple};
use nom::{AsChar, IResult, InputTakeAtPosition};

#[derive(Debug, PartialEq, Eq)]
pub struct URL<'a> {
    protocol: Protocol,
    host: Host,
    port: Option<u16>,
    path: Option<Vec<&'a str>>,
    query: Option<QueryParams<'a>>,
    fragment: Option<&'a str>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Protocol {
    HTTP,
    HTTPS,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Host {
    HOST(String),
    IP([u8; 4]),
}

pub type QueryParam<'a> = (&'a str, &'a str);
pub type QueryParams<'a> = Vec<QueryParam<'a>>;
pub type UrlResult<T, U> = IResult<T, U, VerboseError<T>>;

impl From<&str> for Protocol {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "http://" => Protocol::HTTP,
            "https://" => Protocol::HTTPS,
            _ => unimplemented!("Unsupported Protocol found"),
        }
    }
}
fn main() {
    println!("Hello, world!");
}

fn protocol(input: &str) -> UrlResult<&str, Protocol> {
    context(
        "protocol",
        alt((tag_no_case("HTTP://"), tag_no_case("HTTPS://"))),
    )(input)
    .map(|(next_input, res)| (next_input, res.into()))
}

fn alphanumerichyphen1<T>(input: T) -> UrlResult<T, T>
where
    T: InputTakeAtPosition,
    <T as InputTakeAtPosition>::Item: AsChar,
{
    input.split_at_position1_complete(
        |item| {
            let char_item = item.as_char();
            !(char_item == '-') && !char_item.is_alphanum()
        },
        ErrorKind::AlphaNumeric,
    )
}

fn host(input: &str) -> UrlResult<&str, Host> {
    context(
        "host",
        alt((
            tuple((many1(terminated(alphanumerichyphen1, tag("."))), alpha1)),
            tuple((many_m_n(1, 1, alphanumerichyphen1), take(0 as usize))),
        )),
    )(input)
    .map(|(next_input, mut res)| {
        if !res.1.is_empty() {
            res.0.push(res.1);
        }
        (next_input, Host::HOST(res.0.join(".")))
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use nom::{
        error::{ErrorKind, VerboseError, VerboseErrorKind},
        Err as NomErr,
    };

    #[test]
    fn test_protocol() {
        assert_eq!(protocol("https://yay"), Ok(("yay", Protocol::HTTPS)));
        assert_eq!(protocol("http://yay"), Ok(("yay", Protocol::HTTP)));
        assert_eq!(
            protocol("bla://yay"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    ("bla://yay", VerboseErrorKind::Nom(ErrorKind::Tag)),
                    ("bla://yay", VerboseErrorKind::Nom(ErrorKind::Alt)),
                    ("bla://yay", VerboseErrorKind::Context("protocol")),
                ]
            }))
        );
    }

    #[test]
    fn test_host() {
        assert_eq!(
            host("localhost:3000"),
            Ok((":3000", Host::HOST("localhost".to_string())))
        );
        assert_eq!(
            host("example.com:8080"),
            Ok((":8080", Host::HOST("example.com".to_string())))
        );
        assert_eq!(
            host("subdomain.example.com:8080"),
            Ok((":8080", Host::HOST("subdomain.example.com".to_string())))
        );
        assert_eq!(
            host("$$$.com"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    ("$$$.com", VerboseErrorKind::Nom(ErrorKind::AlphaNumeric)),
                    ("$$$.com", VerboseErrorKind::Nom(ErrorKind::ManyMN)),
                    ("$$$.com", VerboseErrorKind::Nom(ErrorKind::Alt)),
                    ("$$$.com", VerboseErrorKind::Context("host")),
                ]
            }))
        );
        assert_eq!(
            host(".com"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    (".com", VerboseErrorKind::Nom(ErrorKind::AlphaNumeric)),
                    (".com", VerboseErrorKind::Nom(ErrorKind::ManyMN)),
                    (".com", VerboseErrorKind::Nom(ErrorKind::Alt)),
                    (".com", VerboseErrorKind::Context("host")),
                ]
            }))
        );
    }
}
