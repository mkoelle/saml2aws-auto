use super::client;
use aws::xml::{AssumeRoleResponse, AssumeRoleResult, Credentials};
use reqwest::Response;
use serde_xml_rs;
use std::error::Error;
use std::io;

pub fn assume_role(
    arn: &str,
    principal: &str,
    saml_assertion: &str,
    session_duration: Option<i64>,
) -> Result<Credentials, impl Error> {
    let mut res: Response = match client::get_proxied_client_builder()
        .build()
        .unwrap()
        .post("https://sts.amazonaws.com/")
        .query(&[("Version", "2011-06-15"), ("Action", "AssumeRoleWithSAML")])
        .form(&[
            ("PrincipalArn", principal),
            ("RoleArn", arn),
            ("SAMLAssertion", saml_assertion),
            (
                "DurationSeconds",
                &format!("{}", session_duration.or(Some(3600)).unwrap()),
            ),
        ])
        .send()
    {
        Ok(r) => r,
        Err(e) => {
            return Err(io::Error::new(io::ErrorKind::Other, e.description()));
        }
    };

    if res.status() != 200 {
        println!("response: '{:?}'", res.text());
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("sts assume role returned {}", res.status()),
        ));
    }

    let text = res.text().unwrap();

    let response: AssumeRoleResponse = serde_xml_rs::deserialize(text.as_bytes()).unwrap();

    let credentials = (&response.response[0].result)
        .into_iter()
        .find(|r| match r {
            AssumeRoleResult::Credentials(_) => true,
            _ => false,
        })
        .unwrap();

    let credentials = match credentials {
        AssumeRoleResult::Credentials(ref c) => c,
        _ => panic!("this should never, ever happen. It did. awesome."),
    };

    Ok(credentials.clone())
}
