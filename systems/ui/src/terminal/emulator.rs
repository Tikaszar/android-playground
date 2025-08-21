use uuid::Uuid;

pub struct Terminal {
    id: Uuid,
    lines: Vec<String>,
    cursor_position: (usize, usize),
    max_lines: usize,
}

impl Terminal {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            lines: vec![String::new()],
            cursor_position: (0, 0),
            max_lines: 1000,
        }
    }
    
    pub fn write(&mut self, text: String) {
        // Simple terminal emulation
        for ch in text.chars() {
            match ch {
                '\n' => {
                    self.lines.push(String::new());
                    self.cursor_position.0 += 1;
                    self.cursor_position.1 = 0;
                    
                    // Limit lines
                    if self.lines.len() > self.max_lines {
                        self.lines.remove(0);
                        if self.cursor_position.0 > 0 {
                            self.cursor_position.0 -= 1;
                        }
                    }
                }
                '\r' => {
                    self.cursor_position.1 = 0;
                }
                '\t' => {
                    // Tab to next 4-space boundary
                    let spaces = 4 - (self.cursor_position.1 % 4);
                    for _ in 0..spaces {
                        if let Some(line) = self.lines.get_mut(self.cursor_position.0) {
                            line.push(' ');
                        }
                        self.cursor_position.1 += 1;
                    }
                }
                ch => {
                    if let Some(line) = self.lines.get_mut(self.cursor_position.0) {
                        if self.cursor_position.1 >= line.len() {
                            line.push(ch);
                        } else {
                            line.insert(self.cursor_position.1, ch);
                        }
                        self.cursor_position.1 += 1;
                    }
                }
            }
        }
    }
    
    pub fn read(&self) -> Vec<String> {
        self.lines.clone()
    }
}