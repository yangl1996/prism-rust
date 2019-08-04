import json
import sys

template = {
            "Genesis": {
                "NetworkName": "eval",
                "Wallets": []
            },
            "Nodes": []
        }
size = int(sys.argv[1])
stake = 100 // size
last_stake = 100 - stake * (size - 1)

for i in range(size):
    if i != size - 1:
        this_stake = stake
    else:
        this_stake = last_stake
    wallet = {
            "Name": "wallet_{}".format(i),
            "Stake": this_stake,
            "Online": True
            }
    node = {
            "Name": "node_{}".format(i),
            "IsRelay": True,
            "Wallets": [
                {
                    "Name": "wallet_{}".format(i),
                    "ParticipationOnly": False
                }
                ]
            }
    template['Genesis']['Wallets'].append(wallet)
    template['Nodes'].append(node)

print(json.dumps(template, sort_keys=True, indent=4))
