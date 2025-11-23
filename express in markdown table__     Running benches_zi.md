<img src="https://r2cdn.perplexity.ai/pplx-full-logo-primary-dark%402x.png" style="height:64px;margin-right:32px"/>

# express in markdown table

     Running benches/zip_templates_bench.rs (target/release/deps/zip_templates_bench-4b3e190c814a6892)
    Gnuplot not found, using plotters backend

zip_templates::render time: [359.53 ns 360.13 ns 360.85 ns]
change: [−4.1652% −2.7557% −1.0608%] (p = 0.00 < 0.05)
Performance has improved.
Found 12 outliers among 100 measurements (12.00%)
1 (1.00%) high mild
11 (11.00%) high severe

zip_templates::render_flat
time: [36.707 ns 36.787 ns 36.880 ns]
change: [−4.7696% −4.3873% −4.0016%] (p = 0.00 < 0.05)
Performance has improved.
Found 11 outliers among 100 measurements (11.00%)
8 (8.00%) high mild
3 (3.00%) high severe

zip_templates::render_from_vec_smart
time: [21.784 ns 21.823 ns 21.863 ns]
change: [−8.7911% −8.4338% −8.0677%] (p = 0.00 < 0.05)
Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
6 (6.00%) high mild
3 (3.00%) high severe

tera::render time: [1.0891 µs 1.0905 µs 1.0921 µs]
change: [−4.8755% −4.4775% −4.0901%] (p = 0.00 < 0.05)
Performance has improved.
Found 11 outliers among 100 measurements (11.00%)
6 (6.00%) high mild
5 (5.00%) high severe

mystical_runic::render time: [744.88 ns 747.51 ns 750.97 ns]
Found 9 outliers among 100 measurements (9.00%)
2 (2.00%) high mild
7 (7.00%) high severe

simple_replace time: [502.19 ns 503.04 ns 504.08 ns]
change: [−1.6621% −1.2387% −0.8054%] (p = 0.00 < 0.05)
Change within noise threshold.
Found 11 outliers among 100 measurements (11.00%)
4 (4.00%) high mild
7 (7.00%) high severe

simple_replace_flat time: [181.32 ns 181.57 ns 181.86 ns]
change: [+12.015% +12.434% +12.882%] (p = 0.00 < 0.05)
Performance has regressed.
Found 8 outliers among 100 measurements (8.00%)
4 (4.00%) high mild
4 (4.00%) high severe

| Benchmark                              | Time (low) | Time (mid) | Time (high) | Units | Change low | Change mid | Change high | p-value           | Interpretation                | Outliers total | High mild | High severe |
| :------------------------------------- | ---------: | ---------: | ----------: | :---: | ---------: | ---------: | ----------: | :---------------- | :---------------------------- | -------------: | --------: | ----------: |
| `zip_templates::render`                |     359.53 |     360.13 |      360.85 |  ns   |   −4.1652% |   −2.7557% |    −1.0608% | $p = 0.00 < 0.05$ | Performance has improved      |             12 |         1 |          11 |
| `zip_templates::render_flat`           |     36.707 |     36.787 |      36.880 |  ns   |   −4.7696% |   −4.3873% |    −4.0016% | $p = 0.00 < 0.05$ | Performance has improved      |             11 |         8 |           3 |
| `zip_templates::render_from_vec_smart` |     21.784 |     21.823 |      21.863 |  ns   |   −8.7911% |   −8.4338% |    −8.0677% | $p = 0.00 < 0.05$ | Performance has improved      |              9 |         6 |           3 |
| `tera::render`                         |     1089.1 |     1090.5 |      1092.1 |  ns   |   −4.8755% |   −4.4775% |    −4.0901% | $p = 0.00 < 0.05$ | Performance has improved      |             11 |         6 |           5 |
| `mystical_runic::render`               |     744.88 |     747.51 |      750.97 |  ns   |          — |          — |           — | —                 | —                             |              9 |         2 |           7 |
| `simple_replace`                       |     502.19 |     503.04 |      504.08 |  ns   |   −1.6621% |   −1.2387% |    −0.8054% | $p = 0.00 < 0.05$ | Change within noise threshold |             11 |         4 |           7 |
| `simple_replace_flat`                  |     181.32 |     181.57 |      181.86 |  ns   |   +12.015% |   +12.434% |    +12.882% | $p = 0.00 < 0.05$ | Performance has regressed     |              8 |         4 |           4 |
