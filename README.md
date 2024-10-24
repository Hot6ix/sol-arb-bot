# Solana DEX Arbitrage Bot
 

## Objective
* Compare price difference between source pool and destination pool.
* Send transaction and execute swap if exchange profit found.

This project is still working on.

## How does it work?
1. Get all available pool accounts from json files that have various market accounts (e.g. Orca, Raydium, Meteora and else)
2. Fetch all pool account data and put them in a vector. (Run only first time)
3. Fetch all related accounts data from vector declared above and put them in another vector. (Run at intervals)
4. Resolve all available path refer to pool accounts.
5. Run arbitrage once all related accounts data fetched.


## Notice
* Some of dexes have different swap formula, swap result may not be accurate.
* Even if some of dexes have same swap formula, they may have different setting for each pools and different logics, so swap result may not be same.