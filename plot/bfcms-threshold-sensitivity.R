library(ggplot2)
library(stringr)
library(dplyr)
library(readr)
library(patchwork)
library(ggpubr)

source("plot/sensitivity-dataset.R")

options(vsc.rstudioapi = TRUE)
options(vsc.use_httpgd = TRUE)
exports_dir <- "exports/bfcms"

read_export_csv <- function(csv_name, source) {
  read_csv(str_interp("${exports_dir}/${csv_name}")) %>%
    mutate(source = source)
}

df1 <- read_export_csv("threshold.csv", "RT-MD")
df2 <- read_export_csv("threshold-woa.csv", "RT-MD w/o allowlists")

df_all <- bind_rows(df1, df2)
# df_all <- df1

xax = c(0, 40, 80, 120, 160, 200, 240, 280, 320)

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
  labs(
    x = "Threshold (unit/s)",
    y = "TP",
    color = "Dataset"
  ) +
  geom_line(size = 0.4) +
  geom_point(size = 0.6) +
  theme_academic +
  scale_x_continuous(breaks = xax) +
  scale_y_continuous(breaks = sensitivity_y_ticks("bfcms_threshold_tp"))

p_fp <- ggplot(df_all, aes(x = R, y = FP, color = source)) +
  labs(
    x = "Threshold (unit/s)",
    y = "FP",
    color = "Dataset"
  ) +
  geom_line(size = 0.4) +
  geom_point(size = 0.6) +
  theme_academic +
  scale_y_continuous(breaks = sensitivity_y_ticks("bfcms_threshold_fp"), trans=scales::pseudo_log_trans(base = 10)) +
  scale_x_continuous(breaks = xax)

p <- ggarrange(p_fp, p_tp, ncol=2, common.legend = TRUE, legend="top")

show(p)

# par(ask = TRUE)
# plot(p_tp)
# plot(p_fp)
ggsave("plot/bfcms-threshold-fp.pdf", plot = p_fp, width = 7, height = 5, units = "in", dpi = 300)
ggsave("plot/bfcms-threshold-tp.pdf", plot = p_tp, width = 7, height = 5, units = "in", dpi = 300)
ggsave("plot/bfcms-threshold.pdf", plot = p, width = 7, height = 2.5, units = "in", dpi = 300)
# par(ask = FALSE)
