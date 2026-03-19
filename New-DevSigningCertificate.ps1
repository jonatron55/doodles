[CmdletBinding()]
param (
    [Parameter(HelpMessage = "The certificate's common name such as a company, organization, or individual name.")]
    [string] $CommonName,

    [Parameter(HelpMessage = "The city or locality of where the certificate is issued.")]
    [string] $Location,

    [Parameter(HelpMessage = "The country or region of where the certificate is issued, using an ISO 3166-1 alpha-2 country code.")]
    [string] $Country,

    [Parameter(HelpMessage = "The DNS name of the certificate.")]
    [string] $DnsName,

    [Parameter(HelpMessage = "A user-friendly name for the certificate.")]
    [string] $FriendlyName,

    [Parameter(HelpMessage = "The number of years from today for which the certificate will be valid.")]
    [int] $ValidYears
)

$certificate = New-SelfSignedCertificate `
    -Subject "CN=$CommonName, L=$Location, C=$Country" `
    -DnsName $DnsName `
    -CertStoreLocation "Cert:\CurrentUser\My" `
    -FriendlyName $FriendlyName `
    -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3", "2.5.29.19={text}") `
    -KeyUsage DigitalSignature `
    -HashAlgorithm SHA256 `
    -KeyExportPolicy Exportable `
    -NotAfter (Get-Date).AddYears($ValidYears) `
    -ErrorAction Stop

$certificate | Format-List
