use ast::*;
use nom::*;
use super::*;

named!(path<&[u8], String>, map!(recognize!(
    many1!(alt_complete!(alphanumeric | tag!("/")))
), into_string));

named!(pub command<&[u8], Command>, do_parse!(
    path: path >>
    char!(';') >>
    (Command(path))
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha() {
        assert_eq!(
            command(&b"echo;"[..]),
            IResult::Done(&b""[..], Command("echo".to_owned()))
        );
    }

    #[test]
    fn alpha_numeric() {
        assert_eq!(
            command(&b"echo2;"[..]),
            IResult::Done(&b""[..], Command("echo2".to_owned()))
        );
    }

    #[test]
    fn includes_path() {
        assert_eq!(
            command(&b"/bin/echo;"[..]),
            IResult::Done(&b""[..], Command("/bin/echo".to_owned()))
        );
    }

    #[test]
    fn empty_string() {
        assert!(command(&b";"[..]).is_err());
    }
}
