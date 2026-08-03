# Vendored FRED data

A snapshot of economic series from **FRED** (Federal Reserve Bank of St. Louis,
<https://fred.stlouisfed.org>), fetched 2026-08-03 via the open CSV endpoint
(`fredgraph.csv?id=…`, no API key). The `fred` demo ships this snapshot so it runs
offline; press `f` in the app to refresh a series live from FRED.

Every series here is US-federal statistical data — a **work of the U.S. government,
public domain**. FRED aggregates and redistributes it; per FRED's terms it is free to
use, and we credit both FRED and the originating agency.

| File | FRED ID | Series | Source agency | Frequency |
|---|---|---|---|---|
| `unrate.csv` | `UNRATE` | Unemployment Rate | U.S. Bureau of Labor Statistics | monthly |
| `cpi.csv` | `CPIAUCSL` | Consumer Price Index (all items) | U.S. Bureau of Labor Statistics | monthly |
| `gdp.csv` | `GDPC1` | Real Gross Domestic Product | U.S. Bureau of Economic Analysis | quarterly |
| `fedfunds.csv` | `FEDFUNDS` | Federal Funds Effective Rate | Federal Reserve Board | monthly |
| `gs10.csv` | `GS10` | 10-Year Treasury Yield | Federal Reserve Board | monthly |
| `payems.csv` | `PAYEMS` | Total Nonfarm Payrolls | U.S. Bureau of Labor Statistics | monthly |
| `recession.csv` | `USREC` | NBER Recession Indicator (0/1) | National Bureau of Economic Research | monthly |

Only series backed by federal agencies (BLS, BEA, Federal Reserve Board) plus the NBER
recession dates are vendored — deliberately none with redistribution restrictions
(e.g. proprietary index series).
