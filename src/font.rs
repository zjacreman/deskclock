use std::collections::HashMap;

pub struct LargeFont {
    glyphs: HashMap<char, Vec<String>>,
}

impl LargeFont {
    pub fn new() -> Self {
        let mut g = HashMap::new();

        // Helper to insert glyphs
        let mut add = |c: char, glyph: Vec<&str>| {
            g.insert(c, glyph.into_iter().map(|s| s.to_string()).collect());
        };

        // Digits
        add('0', vec![" ███ ", "█   █", "█   █", "█   █", " ███ "]);
        add('1', vec!["  █  ", " ██  ", "  █  ", "  █  ", "  █  "]);
        add('2', vec![" ███ ", "    █", " ███ ", "█    ", " ███ "]);
        add('3', vec![" ███ ", "    █", " ███ ", "    █", " ███ "]);
        add('4', vec!["█   █", "█   █", " ███ ", "    █", "    █"]);
        add('5', vec![" ███ ", "█    ", " ███ ", "    █", " ███ "]);
        add('6', vec![" ███ ", "█    ", " ███ ", "█   █", " ███ "]);
        add('7', vec![" ███ ", "    █", "    █", "    █", "    █"]);
        add('8', vec![" ███ ", "█   █", " ███ ", "█   █", " ███ "]);
        add('9', vec![" ███ ", "█   █", " ███ ", "    █", " ███ "]);
        add(':', vec!["  █  ", "     ", "  █  ", "     ", "  █  "]);
        add(' ', vec!["     ", "     ", "     ", "     ", "     "]);
        add(',', vec!["     ", "     ", "     ", " █   ", " █   "]);

        // Uppercase Alphabet
        add('A', vec![" ███ ", "█   █", " ███ ", "█   █", "█   █"]);
        add('B', vec![" ███ ", "█   █", " ███ ", "█   █", " ███ "]);
        add('C', vec![" ███ ", "█    ", "█    ", "█    ", " ███ "]);
        add('D', vec!["█████", "█   █", "█   █", "█   █", "█████"]);
        add('E', vec![" ███ ", "█    ", " ███ ", "█    ", " ███ "]);
        add('F', vec![" ███ ", "█    ", " ███ ", "█    ", "█    "]);
        add('G', vec![" ███ ", "█    ", " ███ ", "█   █", " ███ "]);
        add('H', vec!["█   █", "█   █", " ███ ", "█   █", "█   █"]);
        add('I', vec![" ███ ", "  █  ", "  █  ", "  █  ", " ███ "]);
        add('J', vec!["    █", "    █", "    █", "█   █", " ███ "]);
        add('K', vec!["█   █", "█  █ ", " ██  ", "█  █ ", "█   █"]);
        add('L', vec!["█    ", "█    ", "█    ", "█    ", " ███ "]);
        add('M', vec!["█   █", "██ ██", "█ █ █", "█   █", "█   █"]);
        add('N', vec!["█   █", "██  █", "█ █ █", "█  ██", "█   █"]);
        add('O', vec![" ███ ", "█   █", "█   █", "█   █", " ███ "]);
        add('P', vec![" ███ ", "█   █", " ███ ", "█    ", "█    "]);
        add('Q', vec![" ███ ", "█   █", " ███ ", "█  █ ", " ███ "]);
        add('R', vec![" ███ ", "█   █", " ███ ", "█  █ ", "█   █"]);
        add('S', vec![" ███ ", "█    ", " ███ ", "    █", " ███ "]);
        add('T', vec![" ███ ", "  █  ", "  █  ", "  █  ", "  █  "]);
        add('U', vec!["█   █", "█   █", "█   █", "█   █", " ███ "]);
        add('V', vec!["█   █", "█   █", " █ █ ", " █ █ ", "  █  "]);
        add('W', vec!["█   █", "█   █", "█ █ █", "██ ██", "█   █"]);
        add('X', vec!["█   █", " █ █ ", "  █  ", " █ █ ", "█   █"]);
        add('Y', vec!["█   █", " █ █ ", "  █  ", "  █  ", "  █  "]);
        add('Z', vec![" ███ ", "    █", "   █ ", "  █  ", " ███ "]);

        Self { glyphs: g }
    }

    pub fn get_glyph(&self, c: char) -> Option<&Vec<String>> {
        self.glyphs.get(&c.to_ascii_uppercase())
    }

    pub fn glyph_width(&self) -> usize {
        5
    }

    pub fn glyph_height(&self) -> usize {
        5
    }
}
