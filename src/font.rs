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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_font_creates_valid_glyphs() {
        let font = LargeFont::new();
        // All digits 0-9 should be present
        for c in '0'..='9' {
            assert!(
                font.get_glyph(c).is_some(),
                "Glyph for digit {} should exist",
                c
            );
        }
    }

    #[test]
    fn test_colon_glyph_exists() {
        let font = LargeFont::new();
        assert!(font.get_glyph(':').is_some(), "Colon glyph should exist");
    }

    #[test]
    fn test_space_glyph_exists() {
        let font = LargeFont::new();
        assert!(font.get_glyph(' ').is_some(), "Space glyph should exist");
    }

    #[test]
    fn test_comma_glyph_exists() {
        let font = LargeFont::new();
        assert!(font.get_glyph(',').is_some(), "Comma glyph should exist");
    }

    #[test]
    fn test_all_uppercase_letters_exist() {
        let font = LargeFont::new();
        for c in 'A'..='Z' {
            assert!(
                font.get_glyph(c).is_some(),
                "Uppercase glyph for {} should exist",
                c
            );
        }
    }

    #[test]
    fn test_lowercase_characters_are_converted_to_uppercase() {
        let font = LargeFont::new();
        // Lowercase letters should be converted to uppercase
        for c in 'a'..='z' {
            let upper = c.to_ascii_uppercase();
            let lower_glyph = font.get_glyph(c);
            let upper_glyph = font.get_glyph(upper);
            assert_eq!(
                lower_glyph, upper_glyph,
                "Lowercase '{}' should map to same glyph as '{}'",
                c, upper
            );
        }
    }

    #[test]
    fn test_unknown_character_returns_none() {
        let font = LargeFont::new();
        assert!(font.get_glyph('@').is_none(), "@ should not have a glyph");
        assert!(font.get_glyph('#').is_none(), "# should not have a glyph");
        assert!(font.get_glyph('$').is_none(), "$ should not have a glyph");
        assert!(font.get_glyph('&').is_none(), "& should not have a glyph");
        assert!(font.get_glyph('%').is_none(), "% should not have a glyph");
    }

    #[test]
    fn test_gdigit_width_is_five() {
        let font = LargeFont::new();
        assert_eq!(font.glyph_width(), 5, "Glyph width should be 5");
    }

    #[test]
    fn test_gdigit_height_is_five() {
        let font = LargeFont::new();
        assert_eq!(font.glyph_height(), 5, "Glyph height should be 5");
    }

    #[test]
    fn test_zero_glyph_content() {
        let font = LargeFont::new();
        let glyph = font.get_glyph('0').unwrap();
        assert_eq!(glyph.len(), 5, "Zero glyph should have 5 rows");
        assert_eq!(glyph[0], " ███ ", "First row of zero should be top border");
        assert_eq!(glyph[1], "█   █", "Second row should have side bars");
        assert_eq!(glyph[2], "█   █", "Third row should have side bars");
        assert_eq!(glyph[3], "█   █", "Fourth row should have side bars");
        assert_eq!(glyph[4], " ███ ", "Fifth row should be bottom border");
    }

    #[test]
    fn test_one_glyph_content() {
        let font = LargeFont::new();
        let glyph = font.get_glyph('1').unwrap();
        assert_eq!(glyph.len(), 5, "One glyph should have 5 rows");
        assert_eq!(glyph[0], "  █  ", "First row of one should be centered");
        assert_eq!(
            glyph[1], " ██  ",
            "Second row of one should have stem right"
        );
    }

    #[test]
    fn test_two_glyph_content() {
        let font = LargeFont::new();
        let glyph = font.get_glyph('2').unwrap();
        assert_eq!(glyph.len(), 5, "Two glyph should have 5 rows");
        assert_eq!(glyph[0], " ███ ", "Top should be horizontal bar");
        assert_eq!(glyph[1], "    █", "Second row should have right stem");
        assert_eq!(glyph[2], " ███ ", "Middle should be horizontal bar");
        assert_eq!(glyph[3], "█    ", "Fourth row should have left stem");
        assert_eq!(glyph[4], " ███ ", "Bottom should be horizontal bar");
    }

    #[test]
    fn test_three_glyph_content() {
        let font = LargeFont::new();
        let glyph = font.get_glyph('3').unwrap();
        assert_eq!(glyph.len(), 5, "Three glyph should have 5 rows");
        assert_eq!(glyph[0], " ███ ", "Top should be horizontal bar");
        assert_eq!(glyph[1], "    █", "Second row should have right stem");
        assert_eq!(glyph[2], " ███ ", "Middle should be horizontal bar");
        assert_eq!(glyph[3], "    █", "Fourth row should have right stem");
        assert_eq!(glyph[4], " ███ ", "Bottom should be horizontal bar");
    }

    #[test]
    fn test_four_glyph_content() {
        let font = LargeFont::new();
        let glyph = font.get_glyph('4').unwrap();
        assert_eq!(glyph.len(), 5, "Four glyph should have 5 rows");
        assert_eq!(glyph[0], "█   █", "Top should have both stems");
        assert_eq!(glyph[1], "█   █", "Second row should have both stems");
        assert_eq!(glyph[2], " ███ ", "Middle should be horizontal bar");
        assert_eq!(glyph[3], "    █", "Fourth row should have right stem");
        assert_eq!(glyph[4], "    █", "Bottom should have right stem");
    }

    #[test]
    fn test_five_glyph_content() {
        let font = LargeFont::new();
        let glyph = font.get_glyph('5').unwrap();
        assert_eq!(glyph.len(), 5, "Five glyph should have 5 rows");
        assert_eq!(glyph[0], " ███ ", "Top should be horizontal bar");
        assert_eq!(glyph[1], "█    ", "Second row should have left stem");
        assert_eq!(glyph[2], " ███ ", "Middle should be horizontal bar");
        assert_eq!(glyph[3], "    █", "Fourth row should have right stem");
        assert_eq!(glyph[4], " ███ ", "Bottom should be horizontal bar");
    }

    #[test]
    fn test_six_glyph_content() {
        let font = LargeFont::new();
        let glyph = font.get_glyph('6').unwrap();
        assert_eq!(glyph.len(), 5, "Six glyph should have 5 rows");
        assert_eq!(glyph[0], " ███ ", "Top should be horizontal bar");
        assert_eq!(glyph[1], "█    ", "Second row should have left stem");
        assert_eq!(glyph[2], " ███ ", "Middle should be horizontal bar");
        assert_eq!(glyph[3], "█   █", "Fourth row should have both stems");
        assert_eq!(glyph[4], " ███ ", "Bottom should be horizontal bar");
    }

    #[test]
    fn test_seven_glyph_content() {
        let font = LargeFont::new();
        let glyph = font.get_glyph('7').unwrap();
        assert_eq!(glyph.len(), 5, "Seven glyph should have 5 rows");
        assert_eq!(glyph[0], " ███ ", "Top should be horizontal bar");
        assert_eq!(glyph[1], "    █", "Second row should have right stem");
        assert_eq!(glyph[2], "    █", "Third row should have right stem");
        assert_eq!(glyph[3], "    █", "Fourth row should have right stem");
        assert_eq!(glyph[4], "    █", "Fifth row should have right stem");
    }

    #[test]
    fn test_eight_glyph_content() {
        let font = LargeFont::new();
        let glyph = font.get_glyph('8').unwrap();
        assert_eq!(glyph.len(), 5, "Eight glyph should have 5 rows");
        assert_eq!(glyph[0], " ███ ", "First row should be horizontal bar");
        assert_eq!(glyph[1], "█   █", "Second row should have side bars");
        assert_eq!(glyph[2], " ███ ", "Third row should be horizontal bar");
        assert_eq!(glyph[3], "█   █", "Fourth row should have side bars");
        assert_eq!(glyph[4], " ███ ", "Fifth row should be horizontal bar");
    }

    #[test]
    fn test_nine_glyph_content() {
        let font = LargeFont::new();
        let glyph = font.get_glyph('9').unwrap();
        assert_eq!(glyph.len(), 5, "Nine glyph should have 5 rows");
        assert_eq!(glyph[0], " ███ ", "First row should be horizontal bar");
        assert_eq!(glyph[1], "█   █", "Second row should have side bars");
        assert_eq!(glyph[2], " ███ ", "Third row should be horizontal bar");
        assert_eq!(glyph[3], "    █", "Fourth row should have right stem");
        assert_eq!(glyph[4], " ███ ", "Fifth row should be horizontal bar");
    }

    #[test]
    fn test_colon_glyph_content() {
        let font = LargeFont::new();
        let glyph = font.get_glyph(':').unwrap();
        assert_eq!(glyph.len(), 5, "Colon glyph should have 5 rows");
        assert_eq!(glyph[0], "  █  ", "First row should have top dot");
        assert_eq!(glyph[1], "     ", "Second row should be empty");
        assert_eq!(glyph[2], "  █  ", "Third row should have middle dot");
        assert_eq!(glyph[3], "     ", "Fourth row should be empty");
        assert_eq!(glyph[4], "  █  ", "Fifth row should have bottom dot");
    }

    #[test]
    fn test_space_glyph_content() {
        let font = LargeFont::new();
        let glyph = font.get_glyph(' ').unwrap();
        assert_eq!(glyph.len(), 5, "Space glyph should have 5 rows");
        for (i, row) in glyph.iter().enumerate() {
            assert_eq!(row, "     ", "Row {} of space should be all spaces", i);
        }
    }

    #[test]
    fn test_comma_glyph_content() {
        let font = LargeFont::new();
        let glyph = font.get_glyph(',').unwrap();
        assert_eq!(glyph.len(), 5, "Comma glyph should have 5 rows");
        assert_eq!(glyph[0], "     ", "First row should be empty");
        assert_eq!(glyph[1], "     ", "Second row should be empty");
        assert_eq!(glyph[2], "     ", "Third row should be empty");
        assert_eq!(glyph[3], " █   ", "Fourth row should have comma top");
        assert_eq!(glyph[4], " █   ", "Fifth row should have comma bottom");
    }

    #[test]
    fn test_a_glyph_content() {
        let font = LargeFont::new();
        let glyph = font.get_glyph('A').unwrap();
        assert_eq!(glyph.len(), 5, "A glyph should have 5 rows");
        assert_eq!(glyph[0], " ███ ", "First row should be horizontal bar");
        assert_eq!(glyph[1], "█   █", "Second row should have side bars");
        assert_eq!(glyph[2], " ███ ", "Third row should be horizontal bar");
        assert_eq!(glyph[3], "█   █", "Fourth row should have side bars");
        assert_eq!(glyph[4], "█   █", "Fifth row should have side bars");
    }

    #[test]
    fn test_b_glyph_content() {
        let font = LargeFont::new();
        let glyph = font.get_glyph('B').unwrap();
        assert_eq!(glyph.len(), 5, "B glyph should have 5 rows");
        assert_eq!(glyph[0], " ███ ", "First row should be horizontal bar");
        assert_eq!(glyph[1], "█   █", "Second row should have side bars");
        assert_eq!(glyph[2], " ███ ", "Third row should be horizontal bar");
        assert_eq!(glyph[3], "█   █", "Fourth row should have side bars");
        assert_eq!(glyph[4], " ███ ", "Fifth row should be horizontal bar");
    }

    #[test]
    fn test_c_glyph_content() {
        let font = LargeFont::new();
        let glyph = font.get_glyph('C').unwrap();
        assert_eq!(glyph.len(), 5, "C glyph should have 5 rows");
        assert_eq!(glyph[0], " ███ ", "First row should be horizontal bar");
        assert_eq!(glyph[1], "█    ", "Second row should have left stem");
        assert_eq!(glyph[2], "█    ", "Third row should have left stem");
        assert_eq!(glyph[3], "█    ", "Fourth row should have left stem");
        assert_eq!(glyph[4], " ███ ", "Fifth row should be horizontal bar");
    }

    #[test]
    fn test_d_glyph_content() {
        let font = LargeFont::new();
        let glyph = font.get_glyph('D').unwrap();
        assert_eq!(glyph.len(), 5, "D glyph should have 5 rows");
        assert_eq!(glyph[0], "█████", "First row should be almost full");
        assert_eq!(glyph[1], "█   █", "Second row should have side bars");
        assert_eq!(glyph[2], "█   █", "Third row should have side bars");
        assert_eq!(glyph[3], "█   █", "Fourth row should have side bars");
        assert_eq!(glyph[4], "█████", "Fifth row should be almost full");
    }

    #[test]
    fn test_m_glyph_content() {
        let font = LargeFont::new();
        let glyph = font.get_glyph('M').unwrap();
        assert_eq!(glyph.len(), 5, "M glyph should have 5 rows");
        assert_eq!(glyph[0], "█   █", "First row should have side bars");
        assert_eq!(glyph[1], "██ ██", "Second row should have middle gap");
        assert_eq!(glyph[2], "█ █ █", "Third row should have middle gaps");
        assert_eq!(glyph[3], "█   █", "Fourth row should have side bars");
        assert_eq!(glyph[4], "█   █", "Fifth row should have side bars");
    }

    #[test]
    fn test_p_glyph_content() {
        let font = LargeFont::new();
        let glyph = font.get_glyph('P').unwrap();
        assert_eq!(glyph.len(), 5, "P glyph should have 5 rows");
        assert_eq!(glyph[0], " ███ ", "First row should be horizontal bar");
        assert_eq!(glyph[1], "█   █", "Second row should have side bars");
        assert_eq!(glyph[2], " ███ ", "Third row should be horizontal bar");
        assert_eq!(glyph[3], "█    ", "Fourth row should have left stem");
        assert_eq!(glyph[4], "█    ", "Fifth row should have left stem");
    }

    #[test]
    fn test_all_glyphs_have_correct_dimensions() {
        let font = LargeFont::new();
        let mut all_chars = Vec::new();
        // Add digits
        for c in '0'..='9' {
            all_chars.push(c);
        }
        // Add colon, space, comma
        all_chars.push(':');
        all_chars.push(' ');
        all_chars.push(',');
        // Add all uppercase letters
        for c in 'A'..='Z' {
            all_chars.push(c);
        }

        for c in all_chars {
            let glyph = font.get_glyph(c);
            assert!(glyph.is_some(), "Glyph for '{}' should exist", c);
            let glyph = glyph.unwrap();
            assert_eq!(glyph.len(), 5, "Glyph for '{}' should have 5 rows", c);
            for (i, row) in glyph.iter().enumerate() {
                assert_eq!(
                    row.chars().count(),
                    5,
                    "Row {} of glyph '{}' should have 5 columns",
                    i,
                    c
                );
            }
        }
    }

    #[test]
    fn test_glyphs_have_correct_characters() {
        let font = LargeFont::new();
        // All rows should only contain valid block character glyphs
        let valid_chars = [' ', '█'];

        let mut all_chars = Vec::new();
        for c in '0'..='9' {
            all_chars.push(c);
        }
        all_chars.push(':');
        all_chars.push(' ');
        all_chars.push(',');
        for c in 'A'..='Z' {
            all_chars.push(c);
        }

        for c in all_chars {
            let glyph = font.get_glyph(c).unwrap();
            for (i, row) in glyph.iter().enumerate() {
                for ch in row.chars() {
                    assert!(
                        valid_chars.contains(&ch),
                        "Invalid character '{}' in row {} of glyph '{}'",
                        ch,
                        i,
                        c
                    );
                }
            }
        }
    }
}
