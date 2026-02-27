package testworkflow

session: {
	beads: [
		{
			id: "bead-001"
			title: "Set up project structure"
			requires: []
			effort: "5min"
			status: "pending"
		},
		{
			id: "bead-002"
			title: "Create base API endpoint"
			requires: ["bead-001"]
			effort: "10min"
			status: "pending"
		},
		{
			id: "bead-003"
			title: "Add authentication middleware"
			requires: ["bead-001"]
			effort: "15min"
			status: "pending"
		},
		{
			id: "bead-004"
			title: "Implement user CRUD operations"
			requires: ["bead-002", "bead-003"]
			effort: "20min"
			status: "pending"
		},
	]
}
