# Prediction Market - Logarithmic Bonding Curve on Anchor


<img width="620" alt="Screenshot 2024-07-16 at 2 46 46 PM" src="https://github.com/user-attachments/assets/60a4a97c-221b-40b0-85f3-732adbcb3ec1">

## Intro

$`reserve = b.ln(e^(supply_x/b)+e^(supply_y/b))`$


This equation has to hold true for all values of supply_x & sypply_y, and the change in reserve caused by the change in supply_y or supply_x, is paid by the user, where the price becomes:


$`price = (reserve - reserve_o)/(supply - supply_o)`$


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
