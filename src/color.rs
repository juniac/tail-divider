use colored::Color;

pub fn parse_color(s: &str) -> Result<Color, String> {
    if s.starts_with('#') && s.len() == 7 {
        let r = u8::from_str_radix(&s[1..3], 16)
            .map_err(|_| format!("invalid hex color: {}", s))?;
        let g = u8::from_str_radix(&s[3..5], 16)
            .map_err(|_| format!("invalid hex color: {}", s))?;
        let b = u8::from_str_radix(&s[5..7], 16)
            .map_err(|_| format!("invalid hex color: {}", s))?;
        Ok(Color::TrueColor { r, g, b })
    } else {
        match s.to_lowercase().as_str() {
            "black" => Ok(Color::Black),
            "red" => Ok(Color::Red),
            "green" => Ok(Color::Green),
            "yellow" => Ok(Color::Yellow),
            "blue" => Ok(Color::Blue),
            "magenta" => Ok(Color::Magenta),
            "cyan" => Ok(Color::Cyan),
            "white" => Ok(Color::White),
            "brightblack" => Ok(Color::BrightBlack),
            "brightred" => Ok(Color::BrightRed),
            "brightgreen" => Ok(Color::BrightGreen),
            "brightyellow" => Ok(Color::BrightYellow),
            "brightblue" => Ok(Color::BrightBlue),
            "brightmagenta" => Ok(Color::BrightMagenta),
            "brightcyan" => Ok(Color::BrightCyan),
            "brightwhite" => Ok(Color::BrightWhite),
            _ => Err(format!(
                "unknown color '{}' — use hex #RRGGBB or a named color (red, brightblue, …)",
                s
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_hex() {
        assert!(parse_color("#FF6600").is_ok());
        assert!(parse_color("#000000").is_ok());
    }

    #[test]
    fn rejects_invalid_hex_digits() {
        assert!(parse_color("#ZZ0000").is_err());
    }

    #[test]
    fn accepts_named_colors_case_insensitive() {
        assert!(parse_color("red").is_ok());
        assert!(parse_color("brightblue").is_ok());
        assert!(parse_color("RED").is_ok());
    }

    #[test]
    fn rejects_unknown_name() {
        assert!(parse_color("chartreuse").is_err());
    }
}
