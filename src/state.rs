use std::time::Instant;

use crate::processor::LineProcessor;

pub enum LogState {
    Active { last_activity: Instant, idle_count: u64 },
    Idle { idle_count: u64 },
}

pub enum Event {
    Line(String),
    Timeout,
    Disconnected,
}

pub enum Action {
    Print(String),
    PrintBlankLine,
    Debug(String),
    Break,
}

pub fn step(state: LogState, event: Event, processor: &LineProcessor) -> (LogState, Vec<Action>) {
    match (state, event) {
        (LogState::Active { last_activity, idle_count }, Event::Line(line)) => {
            if processor.is_activity(&line) {
                (
                    LogState::Active { last_activity: Instant::now(), idle_count },
                    vec![
                        Action::Debug(format!("activity: len={}", line.len())),
                        Action::Print(processor.highlight(&line)),
                    ],
                )
            } else {
                (LogState::Active { last_activity, idle_count }, vec![])
            }
        }
        (LogState::Active { idle_count, .. }, Event::Timeout) => {
            let new_count = idle_count + 1;
            (
                LogState::Idle { idle_count: new_count },
                vec![
                    Action::Debug(format!("idle triggered, count={}", new_count)),
                    Action::Print(processor.format_idle_line(new_count)),
                ],
            )
        }
        (LogState::Idle { idle_count }, Event::Line(line)) => {
            if processor.is_activity(&line) {
                (
                    LogState::Active { last_activity: Instant::now(), idle_count },
                    vec![
                        Action::Debug(format!("activity resumed: len={}", line.len())),
                        Action::PrintBlankLine,
                        Action::Print(processor.highlight(&line)),
                    ],
                )
            } else {
                (LogState::Idle { idle_count }, vec![])
            }
        }
        (state, Event::Disconnected) => (
            state,
            vec![Action::Debug("channel disconnected".to_string()), Action::Break],
        ),
        (state, Event::Timeout) => (state, vec![]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use colored::Color;

    fn processor() -> LineProcessor {
        LineProcessor::new(Color::White)
    }

    #[test]
    fn active_activity_line_prints_and_stays_active() {
        let state = LogState::Active { last_activity: Instant::now(), idle_count: 0 };
        let (next, actions) = step(state, Event::Line("hello world".into()), &processor());
        assert!(matches!(next, LogState::Active { idle_count: 0, .. }));
        assert!(actions.iter().any(|a| matches!(a, Action::Print(_))));
    }

    #[test]
    fn active_non_activity_line_produces_no_actions() {
        let state = LogState::Active { last_activity: Instant::now(), idle_count: 0 };
        let (next, actions) = step(state, Event::Line("   ".into()), &processor());
        assert!(matches!(next, LogState::Active { .. }));
        assert!(actions.is_empty());
    }

    #[test]
    fn active_timeout_transitions_to_idle_and_increments_count() {
        let state = LogState::Active { last_activity: Instant::now(), idle_count: 2 };
        let (next, actions) = step(state, Event::Timeout, &processor());
        assert!(matches!(next, LogState::Idle { idle_count: 3 }));
        assert!(actions.iter().any(|a| matches!(a, Action::Print(_))));
    }

    #[test]
    fn idle_count_accumulates_across_multiple_timeouts() {
        let mut state = LogState::Active { last_activity: Instant::now(), idle_count: 0 };
        for expected in 1..=5 {
            let (next, _) = step(state, Event::Timeout, &processor());
            assert!(matches!(next, LogState::Idle { idle_count } if idle_count == expected));
            state = LogState::Active { last_activity: Instant::now(), idle_count: expected };
        }
    }

    #[test]
    fn idle_activity_line_resumes_to_active_with_blank_line() {
        let state = LogState::Idle { idle_count: 3 };
        let (next, actions) = step(state, Event::Line("hello".into()), &processor());
        assert!(matches!(next, LogState::Active { idle_count: 3, .. }));
        assert!(actions.iter().any(|a| matches!(a, Action::PrintBlankLine)));
        assert!(actions.iter().any(|a| matches!(a, Action::Print(_))));
    }

    #[test]
    fn idle_non_activity_line_stays_idle() {
        let state = LogState::Idle { idle_count: 1 };
        let (next, actions) = step(state, Event::Line("\x1B[0m".into()), &processor());
        assert!(matches!(next, LogState::Idle { idle_count: 1 }));
        assert!(actions.is_empty());
    }

    #[test]
    fn disconnected_emits_break() {
        let state = LogState::Active { last_activity: Instant::now(), idle_count: 0 };
        let (_, actions) = step(state, Event::Disconnected, &processor());
        assert!(actions.iter().any(|a| matches!(a, Action::Break)));
    }
}
