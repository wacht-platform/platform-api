use aws_sdk_sesv2::types::{Body, Content, Destination, EmailContent, Message};
use handlebars::Handlebars;
use std::collections::HashMap;

use crate::{
    application::{AppError, AppState},
    core::queries::{GetEmailTemplateByNameQuery, Query},
};

use super::Command;

pub struct SendEmailCommand {
    deployment_id: i64,
    template_name: String,
    to_email: String,
    variables: HashMap<String, String>,
}

impl SendEmailCommand {
    pub fn new(
        deployment_id: i64,
        template_name: String,
        to_email: String,
        variables: HashMap<String, String>,
    ) -> Self {
        Self {
            deployment_id,
            template_name,
            to_email,
            variables,
        }
    }
}

impl Command for SendEmailCommand {
    type Output = ();

    async fn execute(self, app_state: &AppState) -> Result<Self::Output, AppError> {
        // Get the email template
        let template = GetEmailTemplateByNameQuery::new(self.deployment_id, self.template_name)
            .execute(app_state)
            .await?;

        // Render the template
        let handlebars = Handlebars::new();

        let subject = handlebars
            .render_template(&template.template_subject, &self.variables)
            .map_err(|e| AppError::BadRequest(format!("Failed to render subject: {}", e)))?;

        let body_html = handlebars
            .render_template(&template.template_data, &self.variables)
            .map_err(|e| AppError::BadRequest(format!("Failed to render body: {}", e)))?;

        // Prepare the email
        let from_email = format!("{}@wacht.services", template.template_from);

        let destination = Destination::builder().to_addresses(&self.to_email).build();

        let subject_content = Content::builder()
            .data(subject)
            .charset("UTF-8")
            .build()
            .map_err(|e| AppError::BadRequest(format!("Failed to build subject: {}", e)))?;

        let body_content = Content::builder()
            .data(body_html)
            .charset("UTF-8")
            .build()
            .map_err(|e| AppError::BadRequest(format!("Failed to build body: {}", e)))?;

        let body = Body::builder().html(body_content).build();

        let message = Message::builder()
            .subject(subject_content)
            .body(body)
            .build();

        let email_content = EmailContent::builder().simple(message).build();

        // Send the email using SES client from app_state
        app_state
            .ses_client
            .send_email()
            .from_email_address(from_email)
            .destination(destination)
            .content(email_content)
            .send()
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to send email: {}", e)))?;

        Ok(())
    }
}
