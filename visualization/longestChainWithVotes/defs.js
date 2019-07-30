let defs = svg.append('defs')

// Longest Chain Arrow
defs.append('svg:marker')
    .attr('id', 'longestChain-arrow')
    .attr('refX', 6)
    .attr('refY', 3)
    .attr('markerWidth', 12)
    .attr('markerHeight', 12)
    .attr('markerUnits','userSpaceOnUse')
    .attr('orient', 'auto')
    .append('path')
    .attr('d', 'M 0 0 L 6 3 L 0 6')
    .style('stroke', '#3aafa9')
    .style('stroke-width', 2)
    .style('fill', 'none')
    .style('stroke-opacity', 0.6)

// Vote Arrow
defs.append('svg:marker')
    .attr('id', 'vote-arrow')
    .attr('refX', 6)
    .attr('refY', 3)
    .attr('markerWidth', 12)
    .attr('markerHeight', 12)
    .attr('markerUnits','userSpaceOnUse')
    .attr('orient', 'auto')
    .append('path')
    .attr('d', 'M 0 0 L 6 3 L 0 6')
    .style('stroke-width', 2)
    .style('stroke', '#008080')
    .style('fill', 'none')
    .style('stroke-opacity', 0.6)

// Background
let backgroundGradient = defs.append('linearGradient')
    .attr('id', 'background-gradient')
    .attr('x1', '0%')
    .attr('y1', '0%')
    .attr('x2', '100%')
    .attr('y2', '100%')
    .attr('spreadMethod', 'pad')

backgroundGradient.append('stop')
    .attr('offset', '0%')
    .attr('stop-color', 'black')
    .attr('stop-opacity', 1)

backgroundGradient.append('stop')
    .attr('offset', '100%')
    .attr('stop-color', '#19194d')
    .attr('stop-opacity', 1)

// Linear gradient for votes
let linearGradient = defs.append('linearGradient')
            .attr('id', 'linear-gradient')
            .attr('gradientTransform', 'rotate(0)')

linearGradient.append('stop')
    .attr('offset', '0%')
    .attr('stop-color', 'grey')

linearGradient.append('stop')
    .attr('offset', '100%')
    .attr('stop-color', 'white')

// Blur filter for votes
let blurFilter = defs.append('filter')
    .attr('id','blur')
blurFilter.append('feGaussianBlur')
    .attr('stdDeviation','1')

let glow = (url) => {
    function constructor(svg) {
      let defs = svg.append('defs')
      let filter = defs.append('filter')
          .attr('id', url)
          .attr('x', '-20%')
          .attr('y', '-20%')
          .attr('width', '140%')
          .attr('height', '140%')
        .call(svg => {
          svg.append('feColorMatrix')
              .attr('type', 'matrix')
              .attr('values', colorMatrix)
          svg.append('feGaussianBlur')
               // .attr('in', 'SourceGraphics')
              .attr('stdDeviation', stdDeviation)
              .attr('result', 'coloredBlur')
        })

      filter.append('feMerge')
        .call(svg => {
          svg.append('feMergeNode')
              .attr('in', 'coloredBlur')
          svg.append('feMergeNode')
              .attr('in', 'SourceGraphic')
        })
    }

  constructor.rgb = (value) => {
    rgb = value
    color = d3.rgb(value)
    let matrix = '0 0 0 red 0 0 0 0 0 green 0 0 0 0 blue 0 0 0 1 0'
    colorMatrix = matrix
      .replace('red', color.r)
      .replace('green', color.g)
      .replace('blue', color.b)

    return constructor
  }

  constructor.stdDeviation = (value) => {
    stdDeviation = value
    return constructor
  }

  return constructor
}
