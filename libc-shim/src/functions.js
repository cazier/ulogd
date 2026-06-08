function getServiceLabel(port, protocol = undefined) {
    var service;
    const protocol_name = PROTOCOLS[protocol];

    if (protocol_name !== undefined) {
        service = SERVICES[`${port}/${protocol_name}`];
    }
    if (protocol_name === undefined || service === undefined) {
        service = SERVICES[port];
    }

    if (service) {
        return `${port} (${service})`;
    }
    return `${port}`;
}
