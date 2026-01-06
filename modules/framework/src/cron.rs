pub trait CronJobExecutionSignal: crate::rabbitmq::AmqpMessageSend {
    fn tick(now: time::OffsetDateTime) -> Self;
    fn time_pool(
        now: time::OffsetDateTime,
        last_time: time::OffsetDateTime,
    ) -> std::task::Poll<Self>;
}

pub struct CronSignalSender<T: CronJobExecutionSignal> {
    _marker: std::marker::PhantomData<fn(time::OffsetDateTime) -> T>,
}
