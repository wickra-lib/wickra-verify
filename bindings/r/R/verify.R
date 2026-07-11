#' The wickra-verify library version.
#' @return A version string.
#' @export
wkverify_version <- function() {
  .Call(C_wkverify_version)
}

#' Create a verifier with the default tolerances.
#' @return A `wickra_verify` handle (an external pointer).
#' @export
wkverify_new <- function() {
  .Call(C_wkverify_new)
}

#' Apply a command JSON and return the resulting response JSON.
#' @param verifier A verifier handle from [wkverify_new()].
#' @param cmd_json A command JSON string.
#' @return The response as a JSON string.
#' @export
wkverify_command <- function(verifier, cmd_json) {
  .Call(C_wkverify_command, verifier, cmd_json)
}
