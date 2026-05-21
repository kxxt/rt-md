library(ggplot2)
library(ggpubr)
library(stringr)
library(dplyr)
library(readr)

source("plot/sensitivity-dataset.R")

options(vsc.rstudioapi = TRUE)
options(vsc.use_httpgd = TRUE)
exports_dir <- "exports/uniqd"

read_export_csv <- function(csv_name, source) {
  read_csv(str_interp("${exports_dir}/${csv_name}")) %>%
    mutate(source = source)
}

df1 <- read_export_csv("reset-interval.csv", "w/ allowlists")
# df2 <- read_export_csv("reset-interval-no-peacetime.csv", "w/ popularity allowlist")
# df3 <- read_export_csv("reset-interval-no-popularity.csv", "w/ PT allowlist")
df4 <- read_export_csv("reset-interval-no-allowlist.csv", "w/o allowlists")

df_all <- bind_rows(df1, df4)
df_all["R"] <- df_all["R"] / 1000.
# df_all <- df_all[-1, ] # Drop reset_interval=0 row

# show(df_all)

theme_academic <- theme_bw(base_size = 14, base_family = "serif") +
  theme(
    panel.border = element_rect(color = "black", fill = NA, size = 0.7),
    panel.grid.major = element_line(color = "gray85", size = 0.3),
    panel.grid.minor = element_blank(),
    legend.position = "top",
    legend.title = element_blank(),
    legend.text = element_text(size = 12),
    plot.title = element_text(face = "bold", size = 16, hjust = 0.5),
    axis.title = element_text(face = "bold"),
    # axis.title.y = element_text(vjust = 1.01, angle = 0, hjust=0, margin = margin(r = -15)),
    axis.text = element_text(size = 12, color = "black"),
    legend.key.spacing.x = unit(1, "cm"),
  )

p_tp <- ggplot(df_all, aes(x = R, y = TP, color = source)) +
  geom_line(size = 0.4) +
  geom_point(size = 0.6) +
  theme_academic +
  labs(
    x = "Reset Interval (s)",
    y = "TP",
    color = "Dataset"
  ) +
  scale_y_continuous(
    breaks = sensitivity_y_ticks("reset_interval_tp"),
    limits = sensitivity_y_limits("reset_interval_tp", df_all$TP)
  )

p_fp <- ggplot(df_all, aes(x = R, y = FP, color = source)) + 
  labs(
    x = "Reset Interval (s)",
    y = "FP",
    color = "Dataset"
  ) +
  geom_line(size = 0.4) +
  geom_point(size = 0.6) +
  theme_academic +
  scale_y_continuous(
    breaks = sensitivity_y_ticks("uniqd_reset_interval_fp"),
    limits = sensitivity_y_limits("uniqd_reset_interval_fp", df_all$FP),
    trans=scales::pseudo_log_trans(base = 10)
  )

p <- ggarrange(p_fp, p_tp, ncol=2, common.legend = TRUE, legend="top")

# par(ask = TRUE)
show(p)
# par(ask = FALSE)

ggsave("plot/uniqd-reset-interval-fp.pdf", plot = p_fp, width = 7, height = 5, units = "in", dpi = 300)
ggsave("plot/uniqd-reset-interval-tp.pdf", plot = p_tp, width = 7, height = 5, units = "in", dpi = 300)
ggsave("plot/uniqd-reset-interval.pdf", plot = p, width = 7, height = 3, units = "in", dpi = 300)
