// Declare the chart dimensions and margins.
const width = 928;
const height = 500;
const marginTop = 20;
const marginRight = 30;
const marginBottom = 30;
const marginLeft = 40;

async function chart() {
  const response = await fetch("/api/energy");
  if (!response.ok) {
    throw new Error(`Response status: ${response.status}`);
  }
  const energyResponse = await response.json();
  const data = energyResponse.energy.values;

  // Declare the x (horizontal position) scale.
  const x = d3.scaleTime(
    d3.extent(data, (d) => new Date(d.date)),
    [marginLeft, width - marginRight],
  );

  // Declare the y (vertical position) scale.
  const y = d3.scaleLinear(
    [0, d3.max(data, (d) => d.value)],
    [height - marginBottom, marginTop],
  );

  // Declare the line generator.
  const line = d3
    .line()
    .x((d) => x(new Date(d.date)))
    .y((d) => {
      let v = d.value;
      if (v == null) {
        // no measurement from provider
        v = 0;
      }
      return y(v);
    });

  // Create the SVG container.
  const svg = d3
    .select("body")
    .append("svg")
    .attr("width", width)
    .attr("height", height)
    .attr("viewBox", [0, 0, width, height])
    .attr("style", "max-width: 100%; height: auto; height: intrinsic;");

  // Add the x-axis.
  svg
    .append("g")
    .attr("transform", `translate(0,${height - marginBottom})`)
    .call(
      d3
        .axisBottom(x)
        .ticks(width / 80)
        .tickSizeOuter(0),
    );

  // Add the y-axis, remove the domain line, add grid lines and a label.
  svg
    .append("g")
    .attr("transform", `translate(${marginLeft},0)`)
    .call(d3.axisLeft(y).ticks(height / 48))
    .call((g) => g.select(".domain").remove())
    .call((g) =>
      g
        .selectAll(".tick line")
        .clone()
        .attr("x2", width - marginLeft - marginRight)
        .attr("stroke-opacity", 0.1),
    )
    .call((g) =>
      g
        .append("text")
        .attr("x", -marginLeft)
        .attr("y", 10)
        .attr("fill", "currentColor")
        .attr("text-anchor", "start")
        .text(`energy (${energyResponse.energy.unit})`),
    );

  // Append a path for the line.
  svg
    .append("path")
    .attr("fill", "lightgreen")
    .attr("stroke", "steelblue")
    .attr("stroke-width", 2.0)
    .attr("d", line(energyResponse.energy.values));

  // ============ HOVER FUNCTIONALITY ============

  // Create a group for hover elements
  const hoverGroup = svg
    .append("g")
    .attr("class", "hover-group")
    .style("display", "none");

  // Add vertical line (crosshair)
  const verticalLine = hoverGroup
    .append("line")
    .attr("stroke", "#999")
    .attr("stroke-width", 1)
    .attr("stroke-dasharray", "4,4")
    .attr("y1", marginTop)
    .attr("y2", height - marginBottom);

  // Add circle at intersection point
  const hoverCircle = hoverGroup
    .append("circle")
    .attr("r", 5)
    .attr("fill", "steelblue")
    .attr("stroke", "white")
    .attr("stroke-width", 2);

  // Create tooltip
  const tooltip = d3.select("body").append("div").attr("class", "tooltip");

  // Bisector for finding closest data point
  const bisect = d3.bisector((d) => new Date(d.date)).left;

  // Create invisible overlay for capturing mouse events
  svg
    .append("rect")
    .attr("width", width - marginLeft - marginRight)
    .attr("height", height - marginTop - marginBottom)
    .attr("x", marginLeft)
    .attr("y", marginTop)
    .attr("fill", "none")
    .attr("pointer-events", "all")
    .on("mousemove", function (event) {
      const [mouseX] = d3.pointer(event);

      // Convert mouse x position to date
      const xDate = x.invert(mouseX);

      // Find closest data point
      const index = bisect(data, xDate, 1);
      const d0 = data[index - 1];
      const d1 = data[index];

      // Choose closer point
      const d =
        d1 && xDate - new Date(d0.date) > new Date(d1.date) - xDate ? d1 : d0;

      if (d) {
        const xPos = x(new Date(d.date));
        const yPos = y(d.value || 0);

        // Show hover elements
        hoverGroup.style("display", null);

        // Update vertical line position
        verticalLine.attr("x1", xPos).attr("x2", xPos);

        // Update circle position
        hoverCircle.attr("cx", xPos).attr("cy", yPos);

        // Format date
        const formatDate = d3.timeFormat("%H:%M");
        const formatValue = d3.format(",.2f");

        // Update tooltip
        tooltip
          .style("display", "block")
          .html(
            `
            <strong>${formatDate(new Date(d.date))}</strong><br/>
            ${formatValue(d.value || 0)} ${energyResponse.energy.unit}
          `,
          )
          .style("left", event.pageX + 15 + "px")
          .style("top", event.pageY - 28 + "px");
      }
    })
    .on("mouseout", function () {
      hoverGroup.style("display", "none");
      tooltip.style("display", "none");
    });
}

chart();
