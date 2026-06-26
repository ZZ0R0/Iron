//! [71400] `ironc fmt` (V9.1) : formateur Iron **préservant les commentaires**.
//!
//! Iron est indentation-sensible : un re-spacing token-à-token de l'intérieur des expressions serait
//! fragile (unaire vs binaire, paren d'appel vs de groupe…) et risquerait de changer le sens. Le formateur
//! est donc un **normaliseur de lignes** sûr et idempotent : il recanonise l'**indentation** (multiples de
//! 4 espaces, via une pile d'indentation qui mime le lexer), retire les espaces de fin, réduit les lignes
//! blanches multiples à une seule, et garantit une newline finale unique. Les **commentaires** (`# …`,
//! pleine ligne ou en fin de ligne) sont conservés tels quels — c'est l'avantage du travail à la ligne.
//! Un commentaire pleine ligne s'aligne sur le **code qui suit** (convention Python).

const STEP: usize = 4;

/// Formate une source Iron. Idempotent : `format(format(s)) == format(s)`.
pub fn format(src: &str) -> String {
    let raw: Vec<&str> = src.split('\n').collect();
    let n = raw.len();
    // Passe 1 : profondeur canonique de chaque ligne de **code** ; `None` = blanche/commentaire/continuation.
    let mut depth: Vec<Option<usize>> = vec![None; n];
    let mut cont: Vec<bool> = vec![false; n];
    let mut stack: Vec<usize> = vec![0];
    let mut bracket: i32 = 0;
    for (i, rawline) in raw.iter().enumerate() {
        let line = rawline.trim_end();
        if bracket > 0 {
            // Continuation implicite dans (...)/[...]/{...} (ou bloc `rust { … }`) : préservée verbatim.
            cont[i] = true;
            bracket += delta(line);
            continue;
        }
        let stripped = line.trim_start();
        if stripped.is_empty() || stripped.starts_with('#') {
            continue; // profondeur résolue en passe 2 (lookahead pour les commentaires)
        }
        let phys = line.len() - stripped.len();
        if phys > *stack.last().unwrap() {
            stack.push(phys);
        } else {
            while *stack.last().unwrap() > phys {
                stack.pop();
            }
        }
        depth[i] = Some(stack.len() - 1);
        bracket += delta(stripped);
    }
    // Passe 2 : ré-émission.
    let mut out: Vec<String> = Vec::new();
    let mut pending_blank = false;
    for (i, rawline) in raw.iter().enumerate() {
        let line = rawline.trim_end();
        if cont[i] {
            out.push(line.to_string()); // continuation : indentation d'origine préservée
            continue;
        }
        let stripped = line.trim_start();
        if stripped.is_empty() {
            pending_blank = true;
            continue;
        }
        let d = match depth[i] {
            Some(d) => d,
            // commentaire pleine ligne : profondeur du prochain code, sinon du code précédent, sinon 0.
            None => next_code_depth(&depth, &cont, i)
                .or_else(|| prev_code_depth(&depth, &cont, i))
                .unwrap_or(0),
        };
        if pending_blank && !out.is_empty() {
            out.push(String::new()); // au plus une ligne blanche
        }
        pending_blank = false;
        out.push(format!("{}{}", " ".repeat(STEP * d), stripped));
    }
    // une seule newline finale (retire les blanches de fin).
    while out.last().is_some_and(|l| l.is_empty()) {
        out.pop();
    }
    let mut s = out.join("\n");
    s.push('\n');
    s
}

fn next_code_depth(depth: &[Option<usize>], cont: &[bool], from: usize) -> Option<usize> {
    (from + 1..depth.len()).find_map(|j| if !cont[j] { depth[j] } else { None })
}

fn prev_code_depth(depth: &[Option<usize>], cont: &[bool], from: usize) -> Option<usize> {
    (0..from).rev().find_map(|j| if !cont[j] { depth[j] } else { None })
}

/// Variation nette de profondeur de parenthésage sur une ligne, en ignorant chaînes/char/commentaire.
fn delta(s: &str) -> i32 {
    let b = s.as_bytes();
    let mut d = 0i32;
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'#' => break,
            b'"' => {
                i += 1;
                while i < b.len() && b[i] != b'"' {
                    if b[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            b'\'' => {
                i += 1;
                while i < b.len() && b[i] != b'\'' {
                    if b[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            b'(' | b'[' | b'{' => d += 1,
            b')' | b']' | b'}' => d -= 1,
            _ => {}
        }
        i += 1;
    }
    d
}

#[cfg(test)]
mod tests {
    use super::format;

    #[test]
    fn reindent_to_four_spaces() {
        let src = "def f():\n  x = 1\n  return x\n";
        assert_eq!(format(src), "def f():\n    x = 1\n    return x\n");
    }

    #[test]
    fn preserves_comments_and_aligns_leading() {
        let src = "def f():\n        # doc\n        return 1\n";
        assert_eq!(format(src), "def f():\n    # doc\n    return 1\n");
    }

    #[test]
    fn collapses_blanks_and_trailing_ws() {
        let src = "a = 1   \n\n\n\nb = 2\n\n\n";
        assert_eq!(format(src), "a = 1\n\nb = 2\n");
    }

    #[test]
    fn idempotent() {
        let src = "class P:\n   x: i64\n   def get(self) -> i64:\n      return self.x\n";
        let once = format(src);
        assert_eq!(format(&once), once);
    }

    #[test]
    fn preserves_bracket_continuation() {
        let src = "xs = [\n    1,\n    2,\n]\n";
        assert_eq!(format(src), src); // continuation lines untouched
    }
}
