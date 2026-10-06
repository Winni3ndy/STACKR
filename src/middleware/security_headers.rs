use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use std::future::{ready, Future, Ready};
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct SecurityHeaders;

impl<S, B> Transform<S, ServiceRequest> for SecurityHeaders
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = SecurityHeadersMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SecurityHeadersMiddleware { service }))
    }
}

pub struct SecurityHeadersMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;
            let headers = res.headers_mut();

            headers.insert(
                actix_web::http::header::STRICT_TRANSPORT_SECURITY,
                "max-age=63072000; includeSubDomains; preload"
                    .parse()
                    .unwrap(),
            );
            headers.insert(
                actix_web::http::header::X_CONTENT_TYPE_OPTIONS,
                "nosniff".parse().unwrap(),
            );
            headers.insert(
                actix_web::http::header::X_FRAME_OPTIONS,
                "DENY".parse().unwrap(),
            );
            headers.insert(
                "Permissions-Policy".parse().unwrap(),
                "accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(), usb=()"
                    .parse()
                    .unwrap(),
            );
            headers.insert(
                "X-XSS-Protection".parse().unwrap(),
                "1; mode=block".parse().unwrap(),
            );

            Ok(res)
        })
    }
}
