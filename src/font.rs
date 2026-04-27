use std::collections::HashMap;

pub struct LargeFont {
    glyphs: HashMap<char, Vec<String>>,
}

impl LargeFont {
    pub fn new() -> Self {
        let mut g = HashMap::new();
        
        g.insert('0', vec![
            " ███ ".to_string(),
            "█   █".to_string(),
            "█   █".to_string(),
            "█   █".to_string(),
            " ███ ".to_string(),
        ]);
        g.insert('1', vec![
            "  █  ".to_string(),
            " ██  ".to_string(),
            "  █  ".to_string(),
            "  █  ".to_string(),
            "  █  ".to_string(),
        ]);
        g.insert('2', vec![
            " ███ ".to_string(),
            "    █".to_string(),
            " ███ ".to_string(),
            "█    ".to_string(),
            " ███ ".to_string(),
        ]);
        g.insert('3', vec![
            " ███ ".to_string(),
            "    █".to_string(),
            " ███ ".to_string(),
            "    █".to_string(),
            " ███ ".to_string(),
        ]);
        g.insert('4', vec![
            "█   █".to_string(),
            "█   █".to_string(),
            " ███ ".to_string(),
            "    █".to_string(),
            "    █".to_string(),
        ]);
        g.insert('5', vec![
            " ███ ".to_string(),
            "█    ".to_string(),
            " ███ ".to_string(),
            "    █".to_string(),
            " ███ ".to_string(),
        ]);
        g.insert('6', vec![
            " ███ ".to_string(),
            "█    ".to_string(),
            " ███ ".to_string(),
            "█   █".to_string(),
            " ███ ".to_string(),
        ]);
        g.insert('7', vec![
            " ███ ".to_string(),
            "    █".to_string(),
            "    █".to_string(),
            "    █".to_string(),
            "    █".to_string(),
        ]);
        g.insert('8', vec![
            " ███ ".to_string(),
            "█   █".to_string(),
            " ███ ".to_string(),
            "█   █".to_string(),
            " ███ ".to_string(),
        ]);
        g.insert('9', vec![
            " ███ ".to_string(),
            "█   █".to_string(),
            " ███ ".to_string(),
            "    █".to_string(),
            " ███ ".to_string(),
        ]);
        g.insert(':', vec![
            "  █  ".to_string(),
            "     ".to_string(),
            "  █  ".to_string(),
            "     ".to_string(),
            "  █  ".to_string(),
        ]);
        g.insert(' ', vec![
            "     ".to_string(),
            "     ".to_string(),
            "     ".to_string(),
            "     ".to_string(),
            "     ".to_string(),
        ]);
        g.insert('A', vec![
            " ███ ".to_string(),
            "█   █".to_string(),
            " ███ ".to_string(),
            "█   █".to_string(),
            "█   █".to_string(),
        ]);
        g.insert('P', vec![
            " ███ ".to_string(),
            "█   █".to_string(),
            " ███ ".to_string(),
            "█    ".to_string(),
            "█    ".to_string(),
        ]);
        g.insert('M', vec![
            "█   █".to_string(),
            "██ ██".to_string(),
            "█ █ █".to_string(),
            "█   █".to_string(),
            "█   █".to_string(),
        ]);

        Self { glyphs: g }
    }

    pub fn get_glyph(&self, c: char) -> Option<&Vec<String>> {
        self.glyphs.get(&c)
    }

    pub fn glyph_width(&self) -> usize {
        5
    }

    pub fn glyph_height(&self) -> usize {
        5
    }
}
