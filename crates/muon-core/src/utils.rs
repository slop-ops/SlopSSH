pub fn shell_escape(s: &str) -> String {
    if s.contains(' ')
        || s.contains('"')
        || s.contains('\'')
        || s.contains('$')
        || s.contains('`')
        || s.contains('\\')
        || s.contains('(')
        || s.contains(')')
        || s.contains('&')
        || s.contains('|')
        || s.contains(';')
        || s.contains('<')
        || s.contains('>')
        || s.contains('*')
        || s.contains('?')
        || s.contains('~')
    {
        format!("'{}'", s.replace('\'', "'\\''"))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_escape_simple() {
        assert_eq!(shell_escape("hello"), "hello");
    }

    #[test]
    fn test_shell_escape_spaces() {
        assert_eq!(shell_escape("hello world"), "'hello world'");
    }

    #[test]
    fn test_shell_escape_single_quotes() {
        assert_eq!(shell_escape("it's"), "'it'\\''s'");
    }

    #[test]
    fn test_shell_escape_dollar() {
        assert_eq!(shell_escape("$var"), "'$var'");
    }

    #[test]
    fn test_shell_escape_backtick() {
        assert_eq!(shell_escape("`cmd`"), "'`cmd`'");
    }

    #[test]
    fn test_shell_escape_semicolon() {
        assert_eq!(shell_escape("a;b"), "'a;b'");
    }

    #[test]
    fn test_shell_escape_pipe() {
        assert_eq!(shell_escape("a|b"), "'a|b'");
    }

    #[test]
    fn test_shell_escape_ampersand() {
        assert_eq!(shell_escape("a&b"), "'a&b'");
    }

    #[test]
    fn test_shell_escape_parentheses() {
        assert_eq!(shell_escape("(cmd)"), "'(cmd)'");
    }

    #[test]
    fn test_shell_escape_redirect() {
        assert_eq!(shell_escape(">file"), "'>file'");
    }

    #[test]
    fn test_shell_escape_wildcard() {
        assert_eq!(shell_escape("*.txt"), "'*.txt'");
    }

    #[test]
    fn test_shell_escape_tilde() {
        assert_eq!(shell_escape("~/path"), "'~/path'");
    }

    #[test]
    fn test_shell_escape_backslash() {
        assert_eq!(shell_escape("a\\b"), "'a\\b'");
    }

    #[test]
    fn test_shell_escape_empty() {
        assert_eq!(shell_escape(""), "");
    }

    #[test]
    fn test_shell_escape_path_with_spaces() {
        assert_eq!(
            shell_escape("/path/to/my file.txt"),
            "'/path/to/my file.txt'"
        );
    }
}
