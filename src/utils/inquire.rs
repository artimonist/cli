pub trait CheckInputKey {
    fn is_master(&self) -> bool;
    fn is_mnemonic(&self) -> bool;
}

pub trait InquirePassword {
    fn inquire_password(&mut self, as_salt: bool) -> anyhow::Result<()>;
}

impl CheckInputKey for str {
    #[inline]
    fn is_master(&self) -> bool {
        self.starts_with("xprv") && self.len() == 111
    }

    #[inline]
    fn is_mnemonic(&self) -> bool {
        matches!(self.split_whitespace().count(), 12 | 15 | 18 | 21 | 24)
    }
}

impl InquirePassword for String {
    fn inquire_password(&mut self, as_salt: bool) -> anyhow::Result<()> {
        use super::unicode::UnicodeUtils;
        use inquire::validator::Validation;

        if crate::AUTOMATIC_MODE {
            return Ok(()); // Skip if already set
        }

        const INVALID_MSG: &str = "Encryption key must have at least 5 characters.";
        let validator = |v: &str| {
            if v.unicode_decode().chars().count() < 5 {
                Ok(Validation::Invalid(INVALID_MSG.into()))
            } else {
                Ok(Validation::Valid)
            }
        };

        *self = inquire::Password::new("Encryption Key: ")
            .with_display_mode(inquire::PasswordDisplayMode::Masked)
            .with_display_toggle_enabled()
            .with_custom_confirmation_message("Encryption Key (confirm):")
            .with_custom_confirmation_error_message("The keys don't match.")
            .with_validator(validator)
            .with_formatter(&|_| "Input received".into())
            .with_help_message(if as_salt {
                "Program use encryption key as salt. (Toggle display by CTRL+R)"
            } else {
                "Input encryption key. (Toggle display by CTRL+R)"
            })
            .prompt()?
            .unicode_decode();
        Ok(())
    }
}
