#[allow(dead_code)]
pub trait StyledString {
    fn black(&self) -> String;
    fn red(&self) -> String;
    fn green(&self) -> String;
    fn yellow(&self) -> String;
    fn blue(&self) -> String;
    fn magenta(&self) -> String;
    fn cyan(&self) -> String;
    fn white(&self) -> String;
    fn bold(&self) -> String;
    fn underline(&self) -> String;
    fn normal(&self) -> String;
    fn bright_red(&self) -> String;
}

impl StyledString for str {
    fn black(&self) -> String {
        format!("\x1b[30m{self}\x1b[0m")
    }

    fn red(&self) -> String {
        format!("\x1b[31m{self}\x1b[0m")
    }

    fn green(&self) -> String {
        format!("\x1b[32m{self}\x1b[0m")
    }

    fn yellow(&self) -> String {
        format!("\x1b[33m{self}\x1b[0m")
    }

    fn blue(&self) -> String {
        format!("\x1b[34m{self}\x1b[0m")
    }

    fn magenta(&self) -> String {
        format!("\x1b[35m{self}\x1b[0m")
    }

    fn cyan(&self) -> String {
        format!("\x1b[36m{self}\x1b[0m")
    }

    fn white(&self) -> String {
        format!("\x1b[37m{self}\x1b[0m")
    }

    fn bold(&self) -> String {
        format!("\x1b[1m{self}\x1b[0m")
    }

    fn underline(&self) -> String {
        format!("\x1b[4m{self}\x1b[0m")
    }

    fn normal(&self) -> String {
        format!("\x1b[0m{self}\x1b[0m")
    }

    fn bright_red(&self) -> String {
        format!("\x1b[91m{self}\x1b[0m")
    }
}
