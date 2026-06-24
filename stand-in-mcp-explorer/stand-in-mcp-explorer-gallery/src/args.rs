//! Command-line argument parsing for the gallery Storybook.
//!
//! Contract (compatible with `smoke-open.ps1` / `capture-os.ps1`):
//!   gallery [--capture] `section` `state` `mode`
//!
//! All three positional args are optional. Defaults: `shell` / `overview` / `dark`.
//! With partial args (e.g. a single positional), heuristics:
//!   - If the lone arg is `dark` or `light` → mode; else → section.
//!   - Two args → section + state.
//!
//! Never panics on missing/unknown args — fills in defaults.

pub struct Args {
    pub section: String,
    pub state: String,
    pub mode: String,
    pub capture: bool,
}

impl Args {
    pub fn from_env() -> Self {
        let raw: Vec<String> = std::env::args().skip(1).collect();
        parse_args_from(&raw)
    }
}

pub fn parse_args_from(raw: &[String]) -> Args {
    let (capture, positional) = split_flag(raw, "--capture");

    let (section, state, mode) = match positional.len() {
        0 => (
            String::from("shell"),
            String::from("overview"),
            String::from("dark"),
        ),
        1 => {
            let a = &positional[0];
            if a == "dark" || a == "light" {
                (String::from("shell"), String::from("overview"), a.clone())
            } else {
                (a.clone(), String::from("overview"), String::from("dark"))
            }
        }
        2 => (
            positional[0].clone(),
            positional[1].clone(),
            String::from("dark"),
        ),
        _ => (
            positional[0].clone(),
            positional[1].clone(),
            positional[2].clone(),
        ),
    };

    Args {
        section,
        state,
        mode,
        capture,
    }
}

fn split_flag(args: &[String], flag: &str) -> (bool, Vec<String>) {
    let mut capture = false;
    let mut positional = Vec::new();
    for a in args {
        if a == flag {
            capture = true;
        } else {
            positional.push(a.clone());
        }
    }
    (capture, positional)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn test_all_defaults() {
        let a = parse_args_from(&[]);
        assert_eq!(a.section, "shell");
        assert_eq!(a.state, "overview");
        assert_eq!(a.mode, "dark");
        assert!(!a.capture);
    }

    #[test]
    fn test_capture_flag() {
        let a = parse_args_from(&s(&["--capture"]));
        assert!(a.capture);
        assert_eq!(a.section, "shell");
        assert_eq!(a.mode, "dark");
    }

    #[test]
    fn test_full_args() {
        let a = parse_args_from(&s(&["core", "hover", "light"]));
        assert_eq!(a.section, "core");
        assert_eq!(a.state, "hover");
        assert_eq!(a.mode, "light");
        assert!(!a.capture);
    }

    #[test]
    fn test_full_args_with_capture() {
        let a = parse_args_from(&s(&["--capture", "foundations", "connected", "dark"]));
        assert!(a.capture);
        assert_eq!(a.section, "foundations");
        assert_eq!(a.state, "connected");
        assert_eq!(a.mode, "dark");
    }

    #[test]
    fn test_single_mode_arg() {
        let a = parse_args_from(&s(&["light"]));
        assert_eq!(a.section, "shell");
        assert_eq!(a.state, "overview");
        assert_eq!(a.mode, "light");
    }

    #[test]
    fn test_single_section_arg() {
        let a = parse_args_from(&s(&["core"]));
        assert_eq!(a.section, "core");
        assert_eq!(a.state, "overview");
        assert_eq!(a.mode, "dark");
    }

    #[test]
    fn test_two_args() {
        let a = parse_args_from(&s(&["tools", "selected"]));
        assert_eq!(a.section, "tools");
        assert_eq!(a.state, "selected");
        assert_eq!(a.mode, "dark");
    }
}
