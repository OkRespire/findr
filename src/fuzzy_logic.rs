pub fn fuzzy_matcher(query: &str, candidate: &str) -> Option<i32> {
    let mut score = 0;
    let mut streak = 0;
    let binding = query.to_ascii_lowercase();
    let mut q = binding.chars();
    let mut current = q.next();

    for (i, c) in candidate.to_ascii_lowercase().chars().enumerate() {
        if let Some(qc) = current {
            if qc == c {
                let mut temp_score = 10 + streak * 5 - i as i32;
                if i == 0 {
                    // allow first letter to be the most important
                    temp_score += 15;
                } else if let Some(prev_char) = candidate.chars().nth(i - 1) {
                    if ['_', '-', '/', '.', '\\'].contains(&prev_char) {
                        temp_score += 10;
                    }
                }

                score += temp_score;
                streak += 1;
                current = q.next();
            } else {
                streak = 0;
            }
        }
    }

    if current.is_none() {
        Some(score.max(0))
    } else {
        None
    }
}
