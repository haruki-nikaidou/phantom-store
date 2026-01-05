use kanau::processor::Processor;
use std::fmt::Display;
use time::PrimitiveDateTime;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use tracing::info;

#[derive(Debug)]
pub struct BatchedJobs<T: ScheduledJob> {
    pub jobs: Box<[T]>,
    pub timestamp: PrimitiveDateTime,
}

#[derive(Debug, Clone)]
pub struct JobCompleteSignal<Id> {
    pub id: Id,
    pub complete_time: PrimitiveDateTime,
}

/// Type alias for job execution result
pub type JobResult<Id> = Result<JobCompleteSignal<Id>, crate::Error>;

pub trait OneGoScheduledJob: Sized {
    /// After each `CLOCK_LOOPS` intervals, the job will be executed
    const CLOCK_LOOPS: u64 = 1;
    type Executor: Processor<Self, JobResult<()>> + Send;
    type Scanner: Processor<PrimitiveDateTime, Result<Self, crate::Error>> + Send;
}

pub trait ScheduledJob: Sized {
    /// After each `CLOCK_LOOPS` intervals, the job will be executed
    const CLOCK_LOOPS: u64 = 1;
    /// The executor that execute the job by batch
    type Executor: Processor<BatchedJobs<Self>, ReceiverStream<JobResult<Self::Id>>> + Send;

    /// The scanner that read the jobs from database or kv store
    type Scanner: Processor<PrimitiveDateTime, Result<Box<[Self]>, crate::Error>> + Send;

    /// The tracer that record the job is successful
    type Tracer: Processor<JobResult<Self::Id>, Result<(), crate::Error>> + Send + Sync;

    /// The id of the job that can identify the job and tracing the status
    type Id: Clone + Send + 'static;

    /// get the id of the job
    fn id(&self) -> Self::Id;

    /// execute one job
    fn execute(
        self,
        executor: &Self::Executor,
        time: PrimitiveDateTime,
    ) -> impl Future<Output = ReceiverStream<JobResult<Self::Id>>> + Send {
        executor.process(BatchedJobs {
            jobs: vec![self].into_boxed_slice(),
            timestamp: time,
        })
    }
}

#[derive(Debug, Clone, Copy)]
/// An empty tracer that always returns success and do nothing.
pub struct EmptyTracer;

impl<Id: Clone + Send + Display> Processor<JobResult<Id>, Result<(), crate::Error>>
    for EmptyTracer
{
    async fn process(&self, result: JobResult<Id>) -> Result<(), crate::Error> {
        match result {
            Ok(signal) => {
                tracing::info!(
                    "Job (ID: {}) completed at {}",
                    signal.id,
                    signal.complete_time
                )
            }
            Err(e) => tracing::error!("{e}"),
        }
        Ok(())
    }
}

pub async fn cron_one_go<T: OneGoScheduledJob>(
    scanner: &T::Scanner,
    executor: &T::Executor,
    now: PrimitiveDateTime,
) -> Result<(), crate::Error> {
    let jobs = scanner.process(now).await?;
    executor.process(jobs).await?;
    info!(monotonic_counter.cron_execute = 1);
    Ok(())
}

pub async fn cron<T: ScheduledJob>(
    scanner: &T::Scanner,
    executor: &T::Executor,
    tracer: &'static T::Tracer,
    now: PrimitiveDateTime,
) -> Result<(), crate::Error> {
    let jobs = scanner.process(now).await?;
    let batch = BatchedJobs {
        jobs,
        timestamp: now,
    };
    let mut execute_result_stream = executor.process(batch).await;
    while let Some(execute_result) = execute_result_stream.next().await {
        info!(monotonic_counter.cron_execute = 1);
        tokio::spawn(async move {
            if let Err(e) = tracer.process(execute_result).await {
                tracing::error!("{}", e);
            }
        });
    }
    Ok(())
}
