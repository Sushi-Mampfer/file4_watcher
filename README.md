# File 4 watcher
A script that automatically checks the US. Securities and Exchange Commissions(SEC) file 4 publications to see when someone with inside knowledge makes a trade on the stock market and stores it in a database, if the trade was over a certain percentage of the previously owned stock it sends a notification to a webhook

# Usecases
None for us, as you need to be 18 to trade on the stock market.

# How to use
- Either install it on a server `git clone https://github.com/Sushi-Mampfer/file4_watcher && cd file4_watcher` and `cargo run --release` and set the WEBHOOK env to you webhook.
- or join `https://discord.gg/N9j8tehgcp`