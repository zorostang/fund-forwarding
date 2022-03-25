# fund-forwarding
A smart contract that receives any SNIP-20 and distributes it to multiple addresses.

**USE:**

`receive` is called indirectly by calling the SNIP-20 contract using its `send` function, and it will in turn call this. It accepts the tokens to be forwarded.

`register_token` adds another token that can be distributed by this contract. Simply input the address and hash of the smart contract.

`change_distribution` will change how the tokens are divided and among which addresses they are. Using more than 5 decimal places will break this. All percentages must add to 100%.

`change_admin` changes who has editing control and access to `register_token` and `change_distribution`

`query_dist` allows anyone to view how the funds are divided and where they go to.

