# Prediction Market - Logarithmic Bonding Curve on Anchor

## Intro

$`reserve = b.ln(e^(supply_x/b)+e^(supply_y/b))`$


This equation has to hold true for all values of supply_x & sypply_y, and the change in reserve caused by the change in supply_y or supply_x, is paid by the user, where the price becomes:


$`price = (reserve_new - reserve_old)/(supply_new - supply_old)`$


The initial reserve to be filled while creating the market = $`b.ln2`$


<img width="1440" alt="Screenshot 2024-07-15 at 5 37 48 AM" src="https://github.com/user-attachments/assets/a48fd567-e082-4bc5-8907-d38ddc57fec7">



## How to run locally

- Install dependencies
Extract the zip file in your project's directory and run:

```bash
yarn
```

- Build

```bash
anchor build
```

- Test

```bash
anchor test
```

- Run client

```bash
anchor run client
```
