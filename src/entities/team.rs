pub struct TeamDisplayable {
    pub name: String,
    pub colour: String,
    pub emoji: String,
}

impl TeamDisplayable {
    pub fn new(name: String, colour: String, emoji: String) -> TeamDisplayable {
        let emoji = emoji
            .strip_prefix("0x")
            .and_then(|hex| u32::from_str_radix(hex, 16).ok())
            .and_then(|s| char::try_from(s).ok())
            .map(|c| c.to_string())
            .unwrap_or(emoji);

        TeamDisplayable {
            name,
            colour,
            emoji,
        }
    }

    fn twemoji(&self) -> String {
        self.emoji
            .chars()
            .map(|c| format!("{:x}", u32::from(c)))
            .intersperse("-".into())
            .collect()
    }
}
