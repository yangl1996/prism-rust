// helper function to load json using ajax
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

// palette
// 4CECED
// 69D979
// E7E25B
// E65026
// C4282C

// define the canvas
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
			selector: 'edge[type="VoteForProposer"]',
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

// function to handle error of xhr request (does nothing other than logging to console)
function handle_error(xhr) {
	console.log(xhr);
}

// function to handle json payload
function handle_data(data) {
	// cytoscape-dagre ranks the trees left-to-right according to the order the nodes
	// appear in the nodes list. so we insert nodes by chain number

	// add proposer nodes
	for (hash in data['proposer_nodes']) {
		v = data['proposer_nodes'][hash];
		short_hash = hash.substring(58, 64);

		new_node = {
			group: "nodes",
			data: {
				id: hash,
				disp: short_hash,
				type: 'proposer',
			}
		};
		cy.add(new_node);
	}

	// get number of voter chains
	num_voter = data['voter_longest'].length;

	// add nodes by chain number
	for (voter_idx = 0; voter_idx < num_voter; voter_idx++) {
		// filter out nodes on this chain
		nodes_on_this_chain = Object.keys(data['voter_nodes']).reduce(function (filtered, key) {
			if (data['voter_nodes'][key]['chain'] == voter_idx) {
				filtered[key] = data['voter_nodes'][key];
			}
			return filtered;
		}, {});

		// add those voter nodes
		for (hash in nodes_on_this_chain) {
			v = nodes_on_this_chain[hash];
			short_hash = hash.substring(58, 64);

			new_node = {
				group: "nodes",
				data: {
					id: hash,
					disp: short_hash,
					type: 'voter',
				}
			};
			cy.add(new_node);
		}
	}

	// add edges from block to its immediate parent
	// we reverse the parental links so they origins at the parent (instead of children)
	// so that dagre does the correct thing
	for (idx in data['edges']) {
		e = data['edges'][idx];
		if (e['edgetype'] == 'VoterToVoterParent' ||
			e['edgetype'] == 'ProposerToProposerParent') {
			var new_edge = {
				data: {
					target: e['from'],
					source: e['to'],
					type: "ToParent"
				}
			};
			cy.add(new_edge);
		}
	}

	// run the layout with only edges between blocks and immediate parents
	// or there will be too many edges for dagre to determine the tree structure
	cy.layout({
		name: 'dagre',
	}).run();

	// add those votes
	for (idx in data['edges']) {
		e = data['edges'][idx];
		if (e['edgetype'] == "VoterToProposerParentAndVote" ||
			e['edgetype'] == "VoterToProposerVote") {
			new_edge = {
				data: {
					source: e['from'],
					target: e['to'],
					type: "VoteForProposer"
				}
			};
			cy.add(new_edge);
		}
	}
}

loadJSON("http://SERVER_IP_ADDR:SERVER_PORT_NUMBER/blockchain.json", handle_data, handle_error);
