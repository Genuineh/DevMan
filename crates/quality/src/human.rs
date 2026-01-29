//! Human collaboration for quality checks.

use devman_core::{
    HumanReviewResult, HumanReviewSpec, ReviewAnswer, ReviewQuestion, Severity,
    QualityCategory,
};
use std::time::Duration;

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
                // TODO: Implement actual email sending
                Ok(())
            }
            NotificationChannel::Slack { webhook } => {
                tracing::info!(
                    "Sending Slack review request to {}: {}",
                    webhook,
                    message
                );
                // TODO: Implement actual Slack webhook
                Ok(())
            }
            NotificationChannel::Webhook { url } => {
                tracing::info!("Sending webhook to {}: {}", url, message);
                // TODO: Implement actual webhook call
                Ok(())
            }
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
