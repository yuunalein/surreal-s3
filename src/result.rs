use std::{
    error::Error as StdError,
    fmt::{Debug, Display},
};

use anyhow::Error as AnyError;
use aws_sdk_s3::error::ConnectorError;
use aws_smithy_runtime_api::client::result::SdkError as AwsSdkError;

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct Error {
    pub source: AnyError,
    pub kind: ErrorKind,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorKind {
    Main,
    HttpClient,
    AwsSdk,
}

impl<E: Into<AnyError>> From<E> for Error {
    default fn from(value: E) -> Self {
        match value.into().downcast::<ErrorWrapper>() {
            Ok(wrapped) => wrapped.0,
            Err(e) => Self {
                source: e,
                kind: ErrorKind::Main,
            },
        }
    }
}

impl<E, R> From<AwsSdkError<E, R>> for Error
where
    E: StdError + Send + Sync + 'static,
    R: Debug + Send + Sync + 'static,
{
    fn from(value: AwsSdkError<E, R>) -> Self {
        if let AwsSdkError::DispatchFailure(f) = &value
            && f.is_io()
        {
            match value.into_source().map(|e| {
                {
                    {
                        e.downcast::<ConnectorError>()
                            .and_then(|ce| ce.into_source().downcast::<ErrorWrapper>())
                    }
                }
            }) {
                Ok(Ok(wrapped)) => wrapped.0,
                Ok(Err(boxed_error)) => Self {
                    source: AnyError::from_boxed(boxed_error),
                    kind: ErrorKind::AwsSdk,
                },
                Err(aws_error) => Self {
                    source: aws_error.into(),
                    kind: ErrorKind::AwsSdk,
                },
            }
        } else {
            Self {
                source: value.into(),
                kind: ErrorKind::AwsSdk,
            }
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = if self.kind != ErrorKind::Main {
            format_args!("[{:?}] ", self.kind)
        } else {
            format_args!("")
        };

        f.write_fmt(format_args!("{prefix}{:?}", self.source))
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

pub struct ErrorWrapper(Error);

impl Display for ErrorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Debug for ErrorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl std::error::Error for ErrorWrapper {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source.source()
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

pub trait WrapError {
    fn wrap(self, kind: ErrorKind) -> ErrorWrapper;
}

impl<E: Into<Error>> WrapError for E {
    fn wrap(self, kind: ErrorKind) -> ErrorWrapper {
        let mut err = self.into();
        err.kind = kind;

        ErrorWrapper(err)
    }
}
