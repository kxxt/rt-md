selected_sensitivity_dataset <- function() {
  if (nzchar(Sys.getenv("DATASET_MIXED"))) {
    "mixed"
  } else {
    "ziza"
  }
}

sensitivity_y_ticks <- function(name) {
  dataset <- selected_sensitivity_dataset()

  ticks <- list(
    mixed = list(
      uniqd_threshold_tp = c(0, 1, 3, 5, 10, 15, 17, 19),
      uniqd_threshold_fp = c(0, 1, 10, 100, 500, 2000, 10000),
      bfcms_threshold_tp = c(0, 9, 12, 14, 16, 17, 19),
      bfcms_threshold_fp = c(0, 10, 100, 1000, 4000, 10000, 40000),
      reset_interval_tp = c(0, 1, 3, 5, 10, 15, 17, 19),
      reset_interval_fp = c(10, 100, 500, 2000, 10000, 40000),
      uniqd_reset_interval_fp = c(100, 200, 500, 1000, 5000, 10000, 20000),
      bfcms_reset_interval_fp = c(10, 100, 500, 1000, 5000, 10000, 20000, 40000)
    ),
    ziza = list(
      uniqd_threshold_tp = c(0, 1, 2),
      uniqd_threshold_fp = c(0, 1, 10, 100, 1000, 10000, 30000),
      bfcms_threshold_tp = c(0, 1, 2),
      bfcms_threshold_fp = c(0, 1, 10, 100, 1000, 10000, 30000),
      reset_interval_tp = c(0, 1, 2),
      reset_interval_fp = c(1, 10, 100, 1000, 10000, 30000),
      uniqd_reset_interval_fp = c(1, 10, 100, 1000, 10000, 30000),
      bfcms_reset_interval_fp = c(1, 10, 100, 1000, 10000, 30000)
    )
  )

  ticks[[dataset]][[name]]
}

sensitivity_y_limits <- function(name, values) {
  range(c(values, sensitivity_y_ticks(name)), na.rm = TRUE)
}
