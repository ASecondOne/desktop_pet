use std::collections::HashMap;

use crate::command_keywords::{
    ARTISAN_SAD_SUFFIXES, ARTISAN_SERIOUS_SUFFIXES, CARGO_CURIOUS_SUFFIXES, CARGO_HAPPY_SUFFIXES,
    CARGO_SERIOUS_SUFFIXES, COMPOSER_SAD_SUFFIXES, DESTRUCTIVE_COMMANDS, GIT_CURIOUS_SUFFIXES,
    GIT_HAPPY_SUFFIXES, GIT_SERIOUS_SUFFIXES, LOOKING_AROUND_COMMANDS, PHP_COMMANDS,
    PIP_ANGRY_SUFFIXES, POETRY_ANGRY_SUFFIXES, PYTHON_COMMANDS, ROUTINE_COMMANDS, RUST_COMMANDS,
    SERIOUS_COMMANDS, TIME_QUENSTION_KEYWORDS, WRAPPER_COMMANDS,
};
use chrono::{Local, Timelike};
use colored::Colorize;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct HookEvent {
    timestamp: String,
    command_id: String,
    shell_pid: u32,
    tty: String,
    cwd: String,
    command: String,
    kind: HookKind,
    stream: Option<String>,
    text: Option<String>,
    exit_code: Option<i32>,
}

pub struct PetResponce {
    events: Vec<HookEvent>,
    emotion: Emotion,
    action: String,
    messge: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum HookKind {
    Start,
    Output,
    Finish,
}

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
enum Emotion {
    HAPPY,
    SAD,
    ANGRY,
    SERIOUS,
    COURIOUS,
    NEUTRAL,
}

impl Emotion {
    pub fn get_all() -> [Emotion; 6] {
        [
            Emotion::HAPPY,
            Emotion::SAD,
            Emotion::ANGRY,
            Emotion::SERIOUS,
            Emotion::COURIOUS,
            Emotion::NEUTRAL,
        ]
    }
}

impl HookEvent {
    pub fn print_event_debug(event: &HookEvent) {
        let tty_label = event.tty.strip_prefix("/dev/").unwrap_or(&event.tty);

        match event.kind {
            HookKind::Start => {
                let timestamp = &event.timestamp;
                let shell_pid = event.shell_pid;
                let command = &event.command;
                let cwd = &event.cwd;

                println!("{}", "Start".green());

                println!(
                    "[{}] [{}:{}] $ {}    ({})",
                    timestamp, tty_label, shell_pid, command, cwd
                );

                println!();
            }
            HookKind::Output => {
                let stream = event.stream.as_deref().unwrap_or("stdout");
                let text = event.text.as_deref().unwrap_or("");

                let timestamp = &event.timestamp;
                let shell_pid = event.shell_pid;

                println!("{}", "OUTPUT".yellow());

                println!(
                    "[{}] [{}:{}:{}] {}",
                    timestamp, tty_label, shell_pid, stream, text
                );

                println!();
            }
            HookKind::Finish => {
                let exit_code = event.exit_code.unwrap_or_default();

                let timestamp = &event.timestamp;
                let shell_pid = event.shell_pid;
                let command = &event.command;

                println!("{}", "FINISH".red());

                println!(
                    "[{}] [{}:{}] exit {}    {}",
                    timestamp, tty_label, shell_pid, exit_code, command
                );

                println!();
            }
        }
    }

    pub fn is_finish(&self) -> bool {
        matches!(self.kind, HookKind::Finish)
    }

    pub fn command_id(&self) -> &str {
        &self.command_id
    }
}

fn bump_score(scores: &mut HashMap<Emotion, i32>, emotion: Emotion, amount: i32) {
    *scores.entry(emotion).or_insert(0) += amount;
}

fn command_parts(command: &str) -> Vec<&str> {
    command
        .split_whitespace()
        .map(|part| part.rsplit('/').next().unwrap_or(part))
        .collect()
}

fn effective_command_index(parts: &[&str]) -> Option<usize> {
    let mut index = 0;

    while index < parts.len() {
        if parts[index] == "env" {
            index += 1;
            while index < parts.len() && parts[index].contains('=') {
                index += 1;
            }
            continue;
        }

        if WRAPPER_COMMANDS.contains(&parts[index]) {
            index += 1;
            continue;
        }

        return Some(index);
    }

    None
}

fn normalized_keyword_chars(word: &str) -> Vec<char> {
    word.chars()
        .flat_map(|character| character.to_lowercase())
        .filter(|character| character.is_alphanumeric())
        .collect()
}

fn fluffy_keyword_matching(left: &str, right: &str) -> f32 {
    let left = normalized_keyword_chars(left);
    let right = normalized_keyword_chars(right);

    if left.is_empty() && right.is_empty() {
        return 1.0;
    }

    if left.is_empty() || right.is_empty() {
        return 0.0;
    }

    let mut previous_row: Vec<usize> = (0..=right.len()).collect();
    let mut current_row = vec![0; right.len() + 1];

    for (left_index, left_character) in left.iter().enumerate() {
        current_row[0] = left_index + 1;

        for (right_index, right_character) in right.iter().enumerate() {
            let substitution_cost = usize::from(left_character != right_character);
            let insertion = current_row[right_index] + 1;
            let deletion = previous_row[right_index + 1] + 1;
            let substitution = previous_row[right_index] + substitution_cost;

            current_row[right_index + 1] = insertion.min(deletion).min(substitution);
        }

        std::mem::swap(&mut previous_row, &mut current_row);
    }

    let distance = previous_row[right.len()];
    let longest_word = left.len().max(right.len()) as f32;

    (1.0 - distance as f32 / longest_word).max(0.0)
}

fn fluffy_keyword_finding(text: &str) -> Vec<&'static str> {
    const FLUFFY_KEYWORD_THRESHOLD: f32 = 0.75;
    let mut matched_keywords = Vec::new();

    for word in text
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
    {
        if let Some((keyword, _)) = TIME_QUENSTION_KEYWORDS
            .iter()
            .map(|keyword| (*keyword, fluffy_keyword_matching(word, keyword)))
            .filter(|(_, similarity)| *similarity >= FLUFFY_KEYWORD_THRESHOLD)
            .max_by(|left, right| left.1.total_cmp(&right.1))
        {
            if !matched_keywords.contains(&keyword) {
                matched_keywords.push(keyword);
            }
        }
    }

    matched_keywords
}

fn is_time_question(text: &str) -> bool {
    let matched_keywords = fluffy_keyword_finding(text);

    matched_keywords.contains(&"time") || matched_keywords.len() >= TIME_QUENSTION_KEYWORDS.len()
}

fn build_time_response(formatted_time: &str, hour: u32) -> String {
    let mut lines = vec![format!("The current time is {formatted_time} :3")];

    if hour < 5 || hour >= 22 {
        lines.push("It is already pretty late.".to_string());
        lines.push("You should probably get some sleep soon :3".to_string());
    } else if hour < 9 {
        lines.push("It is still early in the morning.".to_string());
        lines.push("Hope the day starts gently for you :3".to_string());
    }

    lines.join("\n")
}

fn attempt_conversation(event: &HookEvent) -> String {
    let text = event.text.as_deref().unwrap_or("");

    if is_time_question(text) {
        let now = Local::now();
        let formatted_time = now.format("%H:%M").to_string();

        return build_time_response(&formatted_time, now.hour());
    }

    "heh".to_string()
}

impl PetResponce {
    pub fn new(events: Vec<HookEvent>) -> PetResponce {
        PetResponce {
            emotion: PetResponce::evaluate_emotion(&events),
            action: PetResponce::evaluate_action(&events),
            messge: PetResponce::evaluate_messege(&events),
            events,
        }
    }

    pub fn events(&self) -> &[HookEvent] {
        &self.events
    }

    pub fn show(&self) -> String {
        let _event_count = self.events().len();
        format!("{:?} {} \n {}", self.emotion, self.action, self.messge)
    }

    fn evaluate_emotion(events: &[HookEvent]) -> Emotion {
        let mut scores: HashMap<Emotion, i32> = HashMap::new();

        for e in Emotion::get_all() {
            scores.insert(e, 0);
        }

        for event in events {
            if let Some(ec) = event.exit_code {
                match ec {
                    0 => bump_score(&mut scores, Emotion::HAPPY, 1),
                    1 => bump_score(&mut scores, Emotion::SAD, 2),
                    130 | 137 | 143 => bump_score(&mut scores, Emotion::ANGRY, 2),
                    _ => {
                        bump_score(&mut scores, Emotion::ANGRY, 1);
                        bump_score(&mut scores, Emotion::SERIOUS, 1);
                    }
                }
            }

            let parts = command_parts(&event.command);
            let Some(command_index) = effective_command_index(&parts) else {
                continue;
            };

            let command = parts[command_index];
            let suffix = parts.get(command_index + 1).copied();
            let suffix2 = parts.get(command_index + 2).copied();

            if LOOKING_AROUND_COMMANDS.contains(&command) {
                bump_score(&mut scores, Emotion::COURIOUS, 2);
            } else if ROUTINE_COMMANDS.contains(&command) {
                bump_score(&mut scores, Emotion::NEUTRAL, 1);
            } else if DESTRUCTIVE_COMMANDS.contains(&command) {
                bump_score(&mut scores, Emotion::ANGRY, 3);
            } else if RUST_COMMANDS.contains(&command) {
                bump_score(&mut scores, Emotion::HAPPY, 2);
            } else if PYTHON_COMMANDS.contains(&command) {
                bump_score(&mut scores, Emotion::ANGRY, 3);
            } else if PHP_COMMANDS.contains(&command) {
                bump_score(&mut scores, Emotion::SAD, 3);
            } else if SERIOUS_COMMANDS.contains(&command) {
                bump_score(&mut scores, Emotion::SERIOUS, 2);
            }

            match command {
                "git" => match suffix {
                    Some(s) if GIT_HAPPY_SUFFIXES.contains(&s) => {
                        bump_score(&mut scores, Emotion::HAPPY, 4);
                    }
                    Some(s) if GIT_SERIOUS_SUFFIXES.contains(&s) => {
                        bump_score(&mut scores, Emotion::SERIOUS, 4);
                    }
                    Some(s) if GIT_CURIOUS_SUFFIXES.contains(&s) => {
                        bump_score(&mut scores, Emotion::COURIOUS, 2);
                    }
                    Some(_) => bump_score(&mut scores, Emotion::SERIOUS, 1),
                    None => bump_score(&mut scores, Emotion::NEUTRAL, 1),
                },
                "cargo" => match suffix {
                    Some(s) if CARGO_HAPPY_SUFFIXES.contains(&s) => {
                        bump_score(&mut scores, Emotion::HAPPY, 3);
                    }
                    Some(s) if CARGO_SERIOUS_SUFFIXES.contains(&s) => {
                        bump_score(&mut scores, Emotion::SERIOUS, 2);
                    }
                    Some(s) if CARGO_CURIOUS_SUFFIXES.contains(&s) => {
                        bump_score(&mut scores, Emotion::COURIOUS, 2);
                    }
                    Some(_) => bump_score(&mut scores, Emotion::SERIOUS, 1),
                    None => bump_score(&mut scores, Emotion::HAPPY, 1),
                },
                "python" | "python3" => {
                    bump_score(&mut scores, Emotion::ANGRY, 1);
                    if suffix == Some("-m") {
                        bump_score(&mut scores, Emotion::SERIOUS, 1);
                        if let Some(module) = suffix2 {
                            if module == "pip" || module == "pytest" {
                                bump_score(&mut scores, Emotion::ANGRY, 2);
                            }
                        }
                    }
                }
                "pip" | "pip3" => {
                    if matches!(suffix, Some(s) if PIP_ANGRY_SUFFIXES.contains(&s)) {
                        bump_score(&mut scores, Emotion::ANGRY, 2);
                    }
                }
                "poetry" => {
                    if matches!(suffix, Some(s) if POETRY_ANGRY_SUFFIXES.contains(&s)) {
                        bump_score(&mut scores, Emotion::ANGRY, 2);
                    } else {
                        bump_score(&mut scores, Emotion::SERIOUS, 1);
                    }
                }
                "composer" => {
                    if matches!(suffix, Some(s) if COMPOSER_SAD_SUFFIXES.contains(&s)) {
                        bump_score(&mut scores, Emotion::SAD, 2);
                    } else {
                        bump_score(&mut scores, Emotion::SERIOUS, 1);
                    }
                }
                "php" => {
                    if suffix == Some("artisan") {
                        bump_score(&mut scores, Emotion::SAD, 2);
                        if matches!(suffix2, Some(s) if ARTISAN_SERIOUS_SUFFIXES.contains(&s)) {
                            bump_score(&mut scores, Emotion::SERIOUS, 2);
                        }
                        if matches!(suffix2, Some(s) if ARTISAN_SAD_SUFFIXES.contains(&s)) {
                            bump_score(&mut scores, Emotion::SAD, 2);
                        }
                    }
                }
                "docker" | "docker-compose" | "kubectl" => {
                    bump_score(&mut scores, Emotion::SERIOUS, 2);
                }
                "ssh" | "scp" | "rsync" | "chmod" | "chown" | "systemctl" => {
                    bump_score(&mut scores, Emotion::SERIOUS, 1);
                }
                _ => {}
            }
        }

        let mut highest_score = 0;
        let mut highest_emotion = Emotion::NEUTRAL;
        for emotion in Emotion::get_all() {
            let score = *scores.get(&emotion).unwrap_or(&0);
            if score > highest_score {
                highest_score = score;
                highest_emotion = emotion;
            }
        }

        highest_emotion
    }

    fn evaluate_action(_events: &[HookEvent]) -> String {
        "WIP".to_string()
    }

    fn evaluate_messege(events: &[HookEvent]) -> String {
        for event in events {
            let parts = command_parts(&event.command);
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "echo" => {
                    if let Some(output_event) = events
                        .iter()
                        .find(|event| matches!(event.kind, HookKind::Output))
                    {
                        return attempt_conversation(output_event);
                    }
                }
                _ => {}
            }
        }

        "HELLO".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_time_response, fluffy_keyword_finding, fluffy_keyword_matching, is_time_question,
    };

    #[test]
    fn fluffy_keyword_matching_scores_small_word_changes_as_close() {
        assert!(fluffy_keyword_matching("What", "Whats") >= 0.8);
    }

    #[test]
    fn fluffy_keyword_matching_scores_unrelated_words_as_distant() {
        assert!(fluffy_keyword_matching("Bread", "Ai") < 0.3);
    }

    #[test]
    fn fluffy_keyword_finding_collects_distinct_time_keywords() {
        let matches = fluffy_keyword_finding("Whats the time right now?");

        assert!(matches.contains(&"what"));
        assert!(matches.contains(&"time"));
    }

    #[test]
    fn is_time_question_requires_more_than_a_lonely_what() {
        assert!(!is_time_question("what bread"));
        assert!(is_time_question("what time is it"));
        assert!(is_time_question("time?"));
    }

    #[test]
    fn build_time_response_adds_late_night_lines() {
        let message = build_time_response("23:48", 23);

        assert!(message.contains("The current time is 23:48 :3"));
        assert!(message.contains("pretty late"));
        assert!(message.contains("sleep soon"));
    }

    #[test]
    fn build_time_response_adds_early_morning_lines() {
        let message = build_time_response("06:15", 6);

        assert!(message.contains("The current time is 06:15 :3"));
        assert!(message.contains("early in the morning"));
        assert!(message.contains("starts gently"));
    }
}
