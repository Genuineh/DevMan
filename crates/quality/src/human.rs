//! Human collaboration for quality checks.

use devman_core::{
    HumanReviewResult, HumanReviewSpec, ReviewAnswer, ReviewQuestion, Severity,
    QualityCategory,
};
use std::time::Duration;
use serde_json::json;

/// Human review service.
pub struct HumanReviewService {
    /// Default review timeout
    default_timeout: Duration,

    /// Notification channel
    notification_channel: NotificationChannel,
}

/// Channel for sending review notifications.
#[derive(Debug, Clone)]
pub enum NotificationChannel {
    /// Email notifications
    Email {
        recipients: Vec<String>,
    },

    /// Slack webhook
    Slack {
        webhook: String,
    },

    /// Generic webhook
    Webhook {
        url: String,
    },

    /// Console logging (for testing)
    Console,
}

impl HumanReviewService {
    /// Create a new human review service.
    pub fn new(notification_channel: NotificationChannel) -> Self {
        Self {
            default_timeout: Duration::from_secs(24 * 60 * 60), // 24 hours
            notification_channel,
        }
    }

    /// Set default timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    /// Send review notification.
    pub async fn send_notification(
        &self,
        spec: &HumanReviewSpec,
        context: &ReviewContext,
    ) -> Result<(), String> {
        let message = self.format_notification(spec, context);

        match &self.notification_channel {
            NotificationChannel::Console => {
                println!("{}", message);
                Ok(())
            }
            NotificationChannel::Email { recipients } => {
                tracing::info!(
                    "Sending email review request to {:?}: {}",
                    recipients,
                    message
                );
                // Email sending requires SMTP configuration
                // For now, log and return success
                // TODO: Implement actual email sending with lettre or similar crate
                Ok(())
            }
            NotificationChannel::Slack { webhook } => {
                self.send_slack_webhook(webhook, &message).await
            }
            NotificationChannel::Webhook { url } => {
                self.send_generic_webhook(url, spec, context).await
            }
        }
    }

    /// Send notification to Slack webhook.
    async fn send_slack_webhook(&self, webhook: &str, message: &str) -> Result<(), String> {
        let client = reqwest::Client::new();
        let payload = json!({
            "text": message,
            "username": "DevMan Quality Bot",
            "icon_emoji": ":white_check_mark:",
        });

        let response = client
            .post(webhook)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to send Slack webhook: {}", e))?;

        if response.status().is_success() {
            tracing::info!("Slack notification sent successfully");
            Ok(())
        } else {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read response".to_string());
            Err(format!("Slack webhook failed: {} - {}", status, body))
        }
    }

    /// Send notification to generic webhook.
    async fn send_generic_webhook(
        &self,
        url: &str,
        spec: &HumanReviewSpec,
        context: &ReviewContext,
    ) -> Result<(), String> {
        let client = reqwest::Client::new();
        let payload = json!({
            "type": "review_request",
            "guide": spec.review_guide,
            "reviewers": spec.reviewers,
            "questions": spec.review_form.iter().map(|q| &q.question).collect::<Vec<_>>(),
            "context": {
                "description": context.description,
                "files": context.files,
                "check_results": context.check_results,
            }
        });

        let response = client
            .post(url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to send webhook: {}", e))?;

        if response.status().is_success() {
            tracing::info!("Webhook notification sent successfully");
            Ok(())
        } else {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read response".to_string());
            Err(format!("Webhook failed: {} - {}", status, body))
        }
    }

    /// Format notification message.
    fn format_notification(&self, spec: &HumanReviewSpec, context: &ReviewContext) -> String {
        format!(
            "📋 Review Request: {}\n\n\
             Context: {}\n\n\
             Guide: {}\n\n\
             Questions:\n{}",
            spec.review_guide,
            context.description,
            spec.review_guide,
            spec.review_form
                .iter()
                .enumerate()
                .map(|(i, q)| format!("{}. {}", i + 1, q.question))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    /// Process review response.
    pub fn process_response(
        &self,
        spec: &HumanReviewSpec,
        answers: Vec<ReviewAnswer>,
    ) -> HumanReviewResult {
        let approved = self.evaluate_review(spec, &answers);

        HumanReviewResult {
            reviewer: "unknown".to_string(),
            reviewed_at: chrono::Utc::now(),
            answers,
            comments: String::new(),
            approved,
        }
    }

    /// Evaluate if review should pass.
    fn evaluate_review(&self, spec: &HumanReviewSpec, answers: &[ReviewAnswer]) -> bool {
        // Check if any required answers are negative
        for (question, answer) in spec.review_form.iter().zip(answers.iter()) {
            if question.required {
                match &answer.answer {
                    devman_core::AnswerValue::YesNo(false) => return false,
                    devman_core::AnswerValue::Rating(r) if *r < 3 => return false,
                    _ => {}
                }
            }
        }

        true
    }
}

/// Context for a review request.
#[derive(Debug, Clone)]
pub struct ReviewContext {
    /// Description of what needs review
    pub description: String,

    /// Related files/changes
    pub files: Vec<String>,

    /// Check results that triggered review
    pub check_results: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use devman_core::{ReviewQuestion, AnswerType, AnswerValue};

    #[test]
    fn test_review_context_creation() {
        let context = ReviewContext {
            description: "Review pricing calculation changes".to_string(),
            files: vec!["src/pricing.rs".to_string(), "tests/pricing_test.rs".to_string()],
            check_results: vec!["Coverage: 85%".to_string()],
        };

        assert_eq!(context.files.len(), 2);
        assert_eq!(context.check_results.len(), 1);
    }

    #[test]
    fn test_human_review_service_creation() {
        let service = HumanReviewService::new(NotificationChannel::Console);
        assert_eq!(service.default_timeout.as_secs(), 24 * 60 * 60);
    }

    #[test]
    fn test_human_review_service_with_timeout() {
        let service = HumanReviewService::new(NotificationChannel::Console)
            .with_timeout(std::time::Duration::from_secs(3600));
        assert_eq!(service.default_timeout.as_secs(), 3600);
    }

    #[test]
    fn test_format_notification() {
        let service = HumanReviewService::new(NotificationChannel::Console);

        let spec = HumanReviewSpec {
            reviewers: vec!["reviewer@example.com".to_string()],
            review_guide: "Check the implementation".to_string(),
            review_form: vec![
                ReviewQuestion {
                    question: "Is the code correct?".to_string(),
                    answer_type: AnswerType::YesNo,
                    required: true,
                },
            ],
            timeout: std::time::Duration::from_secs(3600),
            auto_pass_threshold: None,
        };

        let context = ReviewContext {
            description: "Review changes".to_string(),
            files: vec!["file.rs".to_string()],
            check_results: vec![],
        };

        let message = service.format_notification(&spec, &context);

        assert!(message.contains("Review Request"));
        assert!(message.contains("Is the code correct?"));
        assert!(message.contains("Review changes"));
    }

    #[test]
    fn test_evaluate_review_all_yes() {
        let service = HumanReviewService::new(NotificationChannel::Console);

        let spec = HumanReviewSpec {
            reviewers: vec![],
            review_guide: String::new(),
            review_form: vec![
                ReviewQuestion {
                    question: "Question 1".to_string(),
                    answer_type: AnswerType::YesNo,
                    required: true,
                },
                ReviewQuestion {
                    question: "Question 2".to_string(),
                    answer_type: AnswerType::YesNo,
                    required: true,
                },
            ],
            timeout: std::time::Duration::from_secs(3600),
            auto_pass_threshold: None,
        };

        let answers = vec![
            ReviewAnswer {
                question: "Question 1".to_string(),
                answer: AnswerValue::YesNo(true),
            },
            ReviewAnswer {
                question: "Question 2".to_string(),
                answer: AnswerValue::YesNo(true),
            },
        ];

        assert!(service.evaluate_review(&spec, &answers));
    }

    #[test]
    fn test_evaluate_review_one_no() {
        let service = HumanReviewService::new(NotificationChannel::Console);

        let spec = HumanReviewSpec {
            reviewers: vec![],
            review_guide: String::new(),
            review_form: vec![
                ReviewQuestion {
                    question: "Question 1".to_string(),
                    answer_type: AnswerType::YesNo,
                    required: true,
                },
            ],
            timeout: std::time::Duration::from_secs(3600),
            auto_pass_threshold: None,
        };

        let answers = vec![
            ReviewAnswer {
                question: "Question 1".to_string(),
                answer: AnswerValue::YesNo(false),
            },
        ];

        assert!(!service.evaluate_review(&spec, &answers));
    }

    #[test]
    fn test_evaluate_review_with_rating() {
        let service = HumanReviewService::new(NotificationChannel::Console);

        let spec = HumanReviewSpec {
            reviewers: vec![],
            review_guide: String::new(),
            review_form: vec![
                ReviewQuestion {
                    question: "Rate the quality".to_string(),
                    answer_type: AnswerType::Rating { min: 1, max: 5 },
                    required: true,
                },
            ],
            timeout: std::time::Duration::from_secs(3600),
            auto_pass_threshold: None,
        };

        // Low rating should fail
        let answers_low = vec![
            ReviewAnswer {
                question: "Rate the quality".to_string(),
                answer: AnswerValue::Rating(2),
            },
        ];
        assert!(!service.evaluate_review(&spec, &answers_low));

        // High rating should pass
        let answers_high = vec![
            ReviewAnswer {
                question: "Rate the quality".to_string(),
                answer: AnswerValue::Rating(4),
            },
        ];
        assert!(service.evaluate_review(&spec, &answers_high));
    }

    #[test]
    fn test_process_response() {
        let service = HumanReviewService::new(NotificationChannel::Console);

        let spec = HumanReviewSpec {
            reviewers: vec!["reviewer@example.com".to_string()],
            review_guide: "Review guide".to_string(),
            review_form: vec![
                ReviewQuestion {
                    question: "Is it good?".to_string(),
                    answer_type: AnswerType::YesNo,
                    required: true,
                },
            ],
            timeout: std::time::Duration::from_secs(3600),
            auto_pass_threshold: None,
        };

        let answers = vec![
            ReviewAnswer {
                question: "Is it good?".to_string(),
                answer: AnswerValue::YesNo(true),
            },
        ];

        let result = service.process_response(&spec, answers);

        assert!(result.approved);
        assert!(result.reviewed_at <= chrono::Utc::now());
    }

    #[tokio::test]
    async fn test_send_console_notification() {
        let service = HumanReviewService::new(NotificationChannel::Console);

        let spec = HumanReviewSpec {
            reviewers: vec![],
            review_guide: "Test review".to_string(),
            review_form: vec![],
            timeout: std::time::Duration::from_secs(3600),
            auto_pass_threshold: None,
        };

        let context = ReviewContext {
            description: "Test context".to_string(),
            files: vec![],
            check_results: vec![],
        };

        let result = service.send_notification(&spec, &context).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_notification_channel_variants() {
        let email = NotificationChannel::Email {
            recipients: vec!["test@example.com".to_string()],
        };
        if let NotificationChannel::Email { recipients } = email {
            assert_eq!(recipients.len(), 1);
        } else {
            panic!("Expected Email variant");
        }

        let slack = NotificationChannel::Slack {
            webhook: "https://hooks.slack.com/test".to_string(),
        };
        if let NotificationChannel::Slack { webhook } = slack {
            assert!(webhook.contains("slack.com"));
        } else {
            panic!("Expected Slack variant");
        }

        let webhook = NotificationChannel::Webhook {
            url: "https://example.com/webhook".to_string(),
        };
        if let NotificationChannel::Webhook { url } = webhook {
            assert!(url.contains("example.com"));
        } else {
            panic!("Expected Webhook variant");
        }
    }
}
