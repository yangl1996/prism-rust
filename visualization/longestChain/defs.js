svg.append('svg:defs').append('svg:marker')
    .attr('id', 'small-arrow')
    .attr('refX', 6)
    .attr('refY', 3)
    .attr('markerWidth', 12)
    .attr('markerHeight', 12)
    .attr('markerUnits','userSpaceOnUse')
    .attr('orient', 'auto')
    .append('path')
    .attr('d', 'M 0 0 L 6 3 L 0 6')
    .style('stroke', 'black')
    .style('fill', 'none')

let linearGradient = svg.append('defs')
            .append('linearGradient')
            .attr('id', 'linear-gradient')
            .attr('gradientTransform', 'rotate(0)')

linearGradient.append('stop')
    .attr('offset', '0%')
    .attr('stop-color', 'grey')

linearGradient.append('stop')
    .attr('offset', '100%')
    .attr('stop-color', 'white')

let blurFilter = svg.append('svg:defs').append('filter')
    .attr('id','blur');
blurFilter.append('feGaussianBlur')
    .attr('stdDeviation','1')

let glowFilter = svg.append('svg:defs').append('filter')
    .attr('id','glow');
glowFilter.append('feGaussianBlur')
    .attr('stdDeviation','2')
    .attr('result','coloredBlur');

let feMerge = glowFilter.append('feMerge');
feMerge.append('feMergeNode')
    .attr('in','coloredBlur');
feMerge.append('feMergeNode')
    .attr('in','SourceGraphic');
