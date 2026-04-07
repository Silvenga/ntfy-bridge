### Netdata Configuration

1. Click on the **Space settings** cog (located above your profile icon)
2. Click on the **Alerts & Notifications** tab
3. Click on the **+ Add configuration** button
4. Add the Webhook integration
5. A modal will be presented to you to enter the required details to enable the configuration:

- **Notification settings**
    - Configuration name (optional): A name for your configuration in order to easily refer to it
    - Rooms: A list of Rooms for which you want to be notified
    - Notifications: The notification types you want to receive
- **Integration configuration**
    - Webhook URL: The url of the service that Netdata will send notifications to. In order to keep the communication
      secured, Netdata only accepts HTTPS urls.
    - Extra headers: Optional key-value pairs that you can set to be included in the HTTP requests sent to the webhook
      URL.
    - Authentication Mechanism, Netdata webhook integration supports 3 different authentication mechanisms.
        - Mutual TLS (recommended): Default authentication mechanism used if no other method is selected
        - Basic: The client sends a request with an Authorization header that includes a base64-encoded string in the
          format **username:password**.
        - Bearer: The client sends a request with an Authorization header that includes a **bearer token**.
- **Verification**
    - Token: The Token from the latest **Test notification** received on the webhook endpoint.
        - Click on the **Test** button to receive a notification. Token will be embedded in the payload.

### Webhook service

A webhook service allows your application to receive real-time alerts from Netdata by sending HTTP requests to a
specified URL.

In this section, we'll go over the steps to set up a generic webhook service, including adding headers, and implementing
different types of authorization mechanisms.

#### Netdata webhook integration

Netdata webhook integration service will send alert and reachability notifications to the destination service as soon as
they are detected.

For alert notifications, the content sent to the destination service contains a JSON object with the following
properties:

| field                             | type                            | description                                                               |
|:----------------------------------|:--------------------------------|:--------------------------------------------------------------------------|
| message                           | string                          | A summary message of the alert.                                           |
| alert                             | string                          | The alert the notification is related to.                                 |
| info                              | string                          | Additional info related with the alert.                                   |
| chart                             | string                          | The chart associated with the alert.                                      |
| context                           | string                          | The chart context.                                                        |
| space                             | string                          | The space where the node that raised the alert is assigned.               |
| Rooms                             | object\[object(string,string)\] | Object with list of Rooms names and urls where the node belongs to.       |
| family                            | string                          | Context family.                                                           |
| class                             | string                          | Classification of the alert, e.g. `Error`.                                |
| severity                          | string                          | Alert severity, can be one of `warning`, `critical` or `clear`.           |
| date                              | string                          | Date of the alert in ISO8601 format.                                      |
| duration                          | string                          | Duration the alert has been raised.                                       |
| additional_active_critical_alerts | integer                         | Number of additional critical alerts currently existing on the same node. |
| additional_active_warning_alerts  | integer                         | Number of additional warning alerts currently existing on the same node.  |
| alert_url                         | string                          | Netdata Cloud URL for this alert.                                         |

For reachability notifications, the JSON object will contain the following properties:

| field            | type    | description                                                                                                                   |
|:-----------------|:--------|:------------------------------------------------------------------------------------------------------------------------------|
| message          | string  | A summary message of the reachability alert.                                                                                  |
| url              | string  | Netdata Cloud URL for the host experiencing the reachability alert.                                                           |
| host             | string  | The hostname experiencing the reachability alert.                                                                             |
| severity         | string  | Severity for this notification. If host is reachable, severity will be `info`, if host is unreachable, it will be `critical`. |
| status           | object  | An object with the status information.                                                                                        |
| status.reachable | boolean | `true` if host is reachable, `false` otherwise                                                                                |
| status.text      | string  | Can be `reachable` or `unreachable`                                                                                           |

#### Extra headers

When setting up a webhook service, the user can specify a set of headers to be included in the HTTP requests sent to the
webhook URL.

By default, the following headers will be sent in the HTTP request

|  **Header**  | **Value**        |
|:------------:|------------------|
| Content-Type | application/json |