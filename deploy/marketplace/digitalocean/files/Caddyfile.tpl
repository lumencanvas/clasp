{{DOMAIN}} {
	handle /auth/* {
		reverse_proxy relay:7350
	}

	{{METRICS_HANDLE}}

	handle {
		reverse_proxy relay:7330 {
			health_uri /healthz
			health_port 7360
			health_interval 10s
		}
	}
}
