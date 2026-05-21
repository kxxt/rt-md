library(ggplot2)
library(stringr)
library(dplyr)
library(readr)
library(patchwork)
library(ggpubr)

options(scipen = 999, digits = 7)
options(vsc.rstudioapi = TRUE)
options(vsc.use_httpgd = TRUE)
exports_dir <- "exports"

df <- read_csv(str_interp("${exports_dir}/additive.csv"))

baseline <- df %>%
  filter(Added == "none") %>%
  select(Dataset, AFPR, FP_baseline = FP)

df <- df %>%
  left_join(baseline, by = c("Dataset", "AFPR")) %>%
  mutate(Reduction = FP_baseline - FP) %>%
  filter(Added != "none" & Added != "full") %>%
  filter(AFPR != 0.005) %>%
  mutate(AFPR = as.character(AFPR))

df <- df %>% filter(Added != "internal")

df$Dataset <- dplyr::recode(df$Dataset, !!!c("ziza" = "Ziza Dataset", "gt" = "Mixed Dataset"))

df_large <- df %>% filter(Added == "peacetime" | Added == "popularity")
# df_small <- df %>% filter(Added == "internal" | Added == "rdns")
df_small <- df %>% filter(Added == "rdns")

totals <- df_large %>%
  group_by(Dataset, AFPR) %>%
  summarise(Total_FP_Reduction = sum(Reduction), .groups = "drop")

# df_all <- df_all[-1, ] # Drop reset_interval=0 row
# show(df_all)
afprs <- factor(df$AFPR)

theme_academic <- theme_bw(base_size = 14, base_family = "serif") +
  theme(
    legend.position = "top",
    legend.title = element_blank(),
    legend.text = element_text(size = 12),
    plot.title = element_text(face = "bold", size = 16, hjust = 0.5),
    axis.title = element_text(face = "bold"),
    # axis.title.y = element_text(vjust = 1.01, angle = 0, hjust=0, margin = margin(r = -15)),
    axis.text = element_text(size = 12, color = "black"),
    legend.key.spacing.x = unit(1, "cm"),
  )

p <- ggplot(df, aes(x = factor(AFPR, levels = c("0.01", "0.001", "0.0001")), y = Reduction, fill = Added)) +
  geom_bar(position = "fill", stat = "identity") +
  geom_text(
    data = totals,
    aes(
      x = factor(AFPR, levels = c("0.01", "0.001", "0.0001")), y = 1,
      label = round(Total_FP_Reduction, 1)
    ),
    vjust = -0.5,
    size = 3,
    color = "black",
    inherit.aes = FALSE
  ) +
  facet_wrap(~Dataset, ncol = 2) +
  scale_fill_brewer(palette = "Set2") +
  labs(
    x = "Acceptable FPR",
    y = "Reduction of FP",
    fill = "Components"
  ) +
  theme_minimal(base_size = 14) +
  theme(
    panel.grid.minor = element_blank(),
    strip.text = element_text(face = "bold"),
    plot.title = element_text(hjust = 0.5, face = "bold"),
    legend.position = "right",
    text = element_text(size = 16, family = "serif")
  ) 
show(p)

# par(ask = TRUE)
# plot(p_tp)
# plot(p_fp)
# ggsave("plot/reset-interval-fp.pdf", plot = p_fp, width = 7, height = 5, units = "in", dpi = 300)
# ggsave("plot/reset-interval-tp.pdf", plot = p_tp, width = 7, height = 5, units = "in", dpi = 300)
ggsave("plot/additive.pdf", plot = p, width = 8, height = 4, units = "in", dpi = 300)
# # par(ask = FALSE)

df_heat <- df %>%
  mutate(
    AFPR = factor(AFPR, levels = c("0.01", "0.001", "0.0001")),
    Added = factor(Added, levels = c("peacetime", "popularity", "rdns"))
  )

p_heat <- ggplot(
  df_heat,
  aes(x = AFPR, y = Added, fill = Reduction)
) +
  geom_tile(color = "white", linewidth = 0.4) +
  geom_text(aes(label = round(Reduction, 2)), size = 4) +
  facet_wrap(~ Dataset, ncol = 2) +
  scale_fill_gradient(
    low  = "#fff897",   # very light blue
    high = "#f98e24",   # medium blue (not dark)
    name = "FP Reduction"
  ) +
  labs(
    x = "Acceptable FPR",
    y = "Component"
  ) +
  theme_minimal(base_size = 14) +
  theme(
    strip.text = element_text(face = "bold", size = 14),
    axis.text = element_text(color = "black", size = 12),
    panel.grid = element_blank(),
    legend.position = "right",
    text = element_text(family = "serif")
  )

show(p_heat)

ggsave(
  "plot/additive-heat.pdf",
  plot = p_heat,
  width = 7,
  height = 2.5,
  units = "in",
  dpi = 300
)