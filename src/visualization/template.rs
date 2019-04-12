pub const BLOCKCHAIN_VISUALIZATION: &str = r###"
<!DOCTYPE html>
<html>
	<head>
		<title>Prism</title>
		<script src="/cytoscape.min.js"></script>
	</head>
	<body>
		<div id="cy" style="width: 100%; height: 100%; position: absolute; top: 0px; left: 0px;"></div>
	</body>
	<script>
		function loadJSON(path, success, error)
{
	var xhr = new XMLHttpRequest();
	xhr.onreadystatechange = function()
	{
		if (xhr.readyState === XMLHttpRequest.DONE) {
			if (xhr.status === 200) {
				if (success)
					success(JSON.parse(xhr.responseText));
			} else {
				if (error)
					error(xhr);
			}
		}
	};
	xhr.open("GET", path, true);
	xhr.send();
}

// colors
// 4CECED
// 69D979
// E7E25B
// E65026
// C4282C

var cy = cytoscape({
	container: document.getElementById('cy'), // container to render in,
	style: [ // the stylesheet for the graph
		{
			selector: 'node',
			style: {
				'shape': 'rectangle',
				'width': 'label',
				'height': 'label',
				'text-halign': 'center',
				'text-valign': 'center',
				'background-color': '#AAA',
				'label': 'data(disp)'
			}
		},
		{
			selector: 'node[type="voter"]',
			style: {
				'shape': 'rectangle',
				'width': 'label',
				'height': 'label',
				'text-halign': 'center',
				'text-valign': 'center',
				'background-color': '#4CECED',
				'label': 'data(disp)'
			}
		},
		{
			selector: 'node[type="proposer"]',
			style: {
				'shape': 'rectangle',
				'width': 'label',
				'height': 'label',
				'text-halign': 'center',
				'text-valign': 'center',
				'background-color': '#69D979',
				'label': 'data(disp)'
			}
		},
		{
			selector: 'edge',
			style: {
				'width': 3,
				'line-color': '#ccc',
				'target-arrow-color': '#ccc',
				'target-arrow-shape': 'triangle',
                'curve-style': 'straight'
			}
		},
		{
			selector: 'edge[type="ToParent"]',
			style: {
				'width': 2,
                'arrow-scale': 0.8,
				'line-color': '#C4282C',
				'target-arrow-color': '#C4282C',
				'target-arrow-shape': 'triangle',
                'curve-style': 'straight'
			}
		},
		{
			selector: 'edge[type="VoterToProposerParent"]',
			style: {
				'width': 1,
                'arrow-scale': 0.5,
				'line-color': '#E65026',
				'target-arrow-color': '#E65026',
				'target-arrow-shape': 'triangle',
                'curve-style': 'straight'
			}
		}
	],
});

function handle_error(xhr) {
	console.log(xhr);
}

function get_graph_dim(data) {
	max_width = [];
	for (hash in data['voter_nodes']) {
		v = data['voter_nodes'][hash];
		chain = v['chain'] + 1;
		level = v['level'];
		if (typeof max_width[chain] === "undefined") {
			max_width[chain] = [];
		}
		if (typeof max_width[chain][level] === "undefined") {
			max_width[chain][level] = 0;
		}
		max_width[chain][level] += 1;
	}

	for (hash in data['proposer_nodes']) {
		v = data['proposer_nodes'][hash];
		chain = 0;
		level = v['level'];
		if (typeof max_width[chain] === "undefined") {
			max_width[chain] = [];
		}
		if (typeof max_width[chain][level] === "undefined") {
			max_width[chain][level] = 0;
		}
		max_width[chain][level] += 1;
	}

	return max_width;
}

function handle_data(data) {
	// First, take a pass of all nodes and calculate the number of
	// nodes at each level and each chain, so we know how "fat" each
	// chain is. Then, calculate the center location of each chain.
	// Finally, assign the location of each block.
	
	// calculate the number of levels of each chain
	widths = get_graph_dim(data);
	max_widths = [];
	for (chain_idx in widths) {
		max_widths[chain_idx] = 0;
		for (level_idx in widths[chain_idx]) {
			if (max_widths[chain_idx] < widths[chain_idx][level_idx]) {
				max_widths[chain_idx] = widths[chain_idx][level_idx];
			}
		}
	}
    
	chain_start_pos = [];
	col_so_far = 0;
	for (chain_idx in max_widths) {
		chain_start_pos[chain_idx] = col_so_far;
		col_so_far += max_widths[chain_idx];
	}
    chain_level_last_pos = [];
    
	// add voter nodes
	for (hash in data['voter_nodes']) {
		v = data['voter_nodes'][hash];
		short_hash = hash.substring(58, 64);
        
        /* magic by Lei, DO NOT TOUCH! */
        chain = v['chain'] + 1;
        level = v['level'];
        if (typeof chain_level_last_pos[chain] === "undefined") {
            chain_level_last_pos[chain] = [];
        }
        if (typeof chain_level_last_pos[chain][level] === "undefined") {
            chain_level_last_pos[chain][level] = chain_start_pos[chain] + max_widths[chain] - widths[chain][level];
        }
        
        if (v['status'] == 'OnMainChain' || v['status'] == 'Leader') {
            col = chain_start_pos[chain] + max_widths[chain] - 1;
        }
        else {
            col = chain_level_last_pos[chain][level];
            chain_level_last_pos[chain][level] += 1;
        }
        /* end of magic */
        
		new_node = {
			group: "nodes",
			data: {
				id: hash,
				disp: short_hash,
				// proposer is 0, voter starts from 1
				chain: chain,
				level: level,
				type: 'voter',
				row: level,
				col: col
			}
		};
		cy.add(new_node);
	}

	// add proposer nodes
	for (hash in data['proposer_nodes']) {
		v = data['proposer_nodes'][hash];
		short_hash = hash.substring(58, 64);
        
        /* magic by Lei, DO NOT TOUCH! */
        chain = 0;
        level = v['level'];
        if (typeof chain_level_last_pos[chain] === "undefined") {
            chain_level_last_pos[chain] = [];
        }
        if (typeof chain_level_last_pos[chain][level] === "undefined") {
            chain_level_last_pos[chain][level] = chain_start_pos[chain] + max_widths[chain] - widths[chain][level];
        }
        
        if (v['status'] == 'OnMainChain' || v['status'] == 'Leader') {
            col = chain_start_pos[chain] + max_widths[chain] - 1;
        }
        else {
            col = chain_level_last_pos[chain][level];
            chain_level_last_pos[chain][level] += 1;
        }
        /* end of magic */
        
		new_node = {
			group: "nodes",
			data: {
				id: hash,
				disp: short_hash,
				// proposer is 0, voter starts from 1
				chain: chain,
				level: level,
				type: 'proposer',
                row: level,
                col: col,
			}
		};
		cy.add(new_node);
	}

	// add edges from block to its immediate parent
	for (idx in data['edges']) {
		e = data['edges'][idx];
		if (e['edgetype'] == 'VoterToVoterParent' ||
		    e['edgetype'] == 'ProposerToProposerParent') {
			new_edge = {
				data: {
					source: e['from'],
					target: e['to'],
                    type: "ToParent"
				}
			};
			
		}
        else if (e['edgetype'] == "VoterToProposerParent") {
			new_edge = {
				data: {
					source: e['from'],
					target: e['to'],
                    type: "VoterToProposerParent"
				}
			};
        }
		try {
			cy.add(new_edge);
		}
		catch {
		}
	}

	// run the layout with only edges between blocks and immediate parents
	// or there will be too many edges for cola to determine the tree structure
	cy.layout({
		name: 'grid',
		fit: true,
		avoidOverlap: true,
        position: function(node) {
            return {
                row: node.data("row"),
                col: node.data("col"),
            };
        },
        condense: true,
	}).run();
}

loadJSON("http://SERVER_IP_ADDR:SERVER_PORT_NUMBER/blockchain.json", handle_data, handle_error)
	</script>
</html>
"###;
