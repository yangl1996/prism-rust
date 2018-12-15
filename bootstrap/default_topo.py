# relationship between name and ip address
nodes = {"spider1": "10.0.0.100",
         "spider2": "10.0.0.101",
         "spider3": "10.0.0.102"}

# btcd peer connection. note that it is directional. [a, b] means a tries to connect to b
btcd_connection = [["spider1", "spider2"], ["spider3", "spider1"]]


