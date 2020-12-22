//! Message handlers for `RadManager`

use actix::{Handler, Message, ResponseFuture};
use futures::Future;
use tokio::util::FutureExt;

use witnet_data_structures::radon_report::{RadonReport, ReportContext};
use witnet_rad::{error::RadError, script::RadonScriptExecutionSettings, types::RadonTypes};
use witnet_validations::validations::{
    construct_report_from_clause_result, evaluate_tally_precondition_clause,
    TallyPreconditionClauseResult,
};

use crate::actors::messages::{ResolveRA, RunTally};

use super::RadManager;

impl Handler<ResolveRA> for RadManager {
    type Result = ResponseFuture<RadonReport<RadonTypes>, RadError>;

    fn handle(&mut self, msg: ResolveRA, _ctx: &mut Self::Context) -> Self::Result {
        let timeout = msg.timeout;
        // The result of the RAD aggregation is computed asynchronously, because the async block
        // returns a std future. It is called fut03 because it uses the 0.3 version of futures,
        // while most of our codebase is still on 0.1 futures.
        let fut03 = async {
            let sources = msg.rad_request.retrieve;
            let aggregator = msg.rad_request.aggregate;

            let retrieve_responses_fut = sources
                .iter()
                .map(|retrieve| witnet_rad::run_retrieval(retrieve));

            // Perform retrievals in parallel for the sake of synchronization between sources
            //  (increasing the likeliness of multiple sources returning results that are closer to each
            //  other).
            let retrieve_responses: Vec<RadonReport<RadonTypes>> =
                futures03::future::join_all(retrieve_responses_fut)
                    .await
                    .into_iter()
                    .map(|retrieve| RadonReport::from_result(retrieve, &ReportContext::default()))
                    .collect();

            let clause_result = evaluate_tally_precondition_clause(retrieve_responses, 0.2, 1);

            match clause_result {
                Ok(TallyPreconditionClauseResult::MajorityOfValues {
                    values,
                    liars: _liars,
                    errors: _errors,
                }) => {
                    // Perform aggregation on the values that made it to the output vector after applying the
                    // source scripts (aka _normalization scripts_ in the original whitepaper) and filtering out
                    // failures.
                    witnet_rad::run_aggregation_report(
                        values,
                        &aggregator,
                        RadonScriptExecutionSettings::all_but_partial_results(),
                    )
                }
                Ok(TallyPreconditionClauseResult::MajorityOfErrors { errors_mode }) => {
                    Ok(RadonReport::from_result(
                        Ok(RadonTypes::RadonError(errors_mode)),
                        &ReportContext::default(),
                    ))
                }
                Err(e) => Ok(RadonReport::from_result(Err(e), &ReportContext::default())),
            }
        };

        // Magic conversion from std::future::Future (futures 0.3) and futures::Future (futures 0.1)
        let fut = futures_util::compat::Compat::new(Box::pin(fut03));

        if let Some(timeout) = timeout {
            // Add timeout, if there is one
            // TODO: this timeout only works if there are no blocking operations.
            // Since currently the execution of RADON is blocking this thread, we can only
            // handle HTTP timeouts.
            // A simple fix would be to offload computation to another thread, to avoid blocking
            // the main thread. Then the timeout would apply to the message passing between threads.
            Box::new(fut.timeout(timeout).then(|result| match result {
                Ok(x) => Ok(x),
                Err(error) => {
                    if error.is_elapsed() {
                        Ok(RadonReport::from_result(
                            Err(RadError::RetrieveTimeout),
                            &ReportContext::default(),
                        ))
                    } else if error.is_inner() {
                        Err(error.into_inner().unwrap())
                    } else {
                        panic!("Unhandled tokio timer error");
                    }
                }
            }))
        } else {
            Box::new(fut)
        }
    }
}

impl Handler<RunTally> for RadManager {
    type Result = <RunTally as Message>::Result;

    fn handle(&mut self, msg: RunTally, _ctx: &mut Self::Context) -> Self::Result {
        let packed_script = msg.script;
        let reports = msg.reports;

        let reports_len = reports.len();
        let clause_result =
            evaluate_tally_precondition_clause(reports, msg.min_consensus_ratio, msg.commits_count);

        construct_report_from_clause_result(clause_result, &packed_script, reports_len)
    }
}
