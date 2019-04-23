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
			selector: 'node[type="TransactionBlock"]',
			style: {
				'shape': 'rectangle',
				'width': 'label',
				'height': 'label',
				'text-halign': 'center',
				'text-valign': 'center',
				'background-color': '#FFDDDD',
				'label': 'data(disp)',
			}
		},
		{
			selector: 'node[type="Transaction"]',
			style: {
				'shape': 'ellipse',
				'width': 'label',
				'height': 'label',
				'text-halign': 'center',
				'text-valign': 'center',
				'background-color': '#DDFFDD',
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
				'width': 0,
				'arrow-scale': 0,
				'line-color': '#000000',
				'target-arrow-color': '#000000',
				'target-arrow-shape': 'triangle',
				'curve-style': 'straight'
			}
		},
		{
			selector: 'edge[type="TransactionReference"]',
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
	// add tx block nodes
	for (idx in data['transactions_blocks']) {
		block = data['transactions_blocks'][idx];
		hash = block['block_hash'];
		short_hash = hash.substring(58, 64);
		new_node = {
			group: "nodes",
			data: {
				id: hash,
				disp: short_hash,
				type: "TransactionBlock",
			}
		};
		cy.add(new_node);
		if (idx >0) {
			new_edge = {
				data: {
					source: hash,
					target: data['transactions_blocks'][idx-1]['block_hash'],
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
	cy.elements().lock();
	for (idx in data['transactions_blocks']) {
		block = data['transactions_blocks'][idx];
		hash = block['block_hash'];
		for (idx_ in block['transactions']) {
			transaction = block['transactions'][idx_];
			h = transaction['tx_hash'];
			short_h = h.substring(58, 64);

			new_node = {
				group: "nodes",
				data: {
					id: h,
					disp: short_h,
					type: "Transaction",
				}
			};
			cy.add(new_node);
			new_edge = {
				data: {
					source: hash,
					target: h,
					type: "TransactionReference"
				}
			};
			cy.add(new_edge);
		}
	}
	cy.layout({
		name: 'dagre',
	}).run();
	// cy.filter('node[type = "Transaction"]').layout({
	// 	name: 'dagre',
	// }).run();

}

loadJSON("http://SERVER_IP_ADDR:SERVER_PORT_NUMBER/ledger.json", handle_data, handle_error);
