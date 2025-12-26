# Test script for WASM modules
# Run this after starting the server with: cargo run

$BASE_URL = "http://localhost:3000"

function Submit-Job {
    param(
        [string]$ModuleId,
        [string[]]$Capabilities = @()
    )
    
    $body = @{
        tenant_id = "test-tenant"
        module_id = $ModuleId
        payload = @{}
        capabilities = $Capabilities
    } | ConvertTo-Json
    
    $response = Invoke-RestMethod -Uri "$BASE_URL/jobs" -Method Post -Body $body -ContentType "application/json"
    return $response.job_id
}

function Get-JobResult {
    param([string]$JobId)
    Start-Sleep -Seconds 1
    return Invoke-RestMethod -Uri "$BASE_URL/jobs/$JobId" -Method Get
}

function Get-OutputString {
    param($Result)
    if ($Result.result -and $Result.result.output) {
        # output is an array of bytes, convert to string
        $bytes = [byte[]]$Result.result.output
        return [System.Text.Encoding]::UTF8.GetString($bytes)
    }
    return "N/A"
}

Write-Host "=== Testing WASM Modules ===" -ForegroundColor Cyan
Write-Host ""

# Test 1: Simple compute (no capabilities) - should return 60
Write-Host "1. Testing simple-compute (no capabilities)..." -ForegroundColor Yellow
try {
    $jobId = Submit-Job -ModuleId "simple-compute"
    Write-Host "   Job ID: $jobId" -ForegroundColor Gray
    $result = Get-JobResult -JobId $jobId
    if ($result.status.finished -ne $null) {
        $output = Get-OutputString -Result $result
        Write-Host "   Result: $output (expected: 60)" -ForegroundColor Green
    } else {
        Write-Host "   FAILED: $($result.status.failed)" -ForegroundColor Red
    }
} catch {
    Write-Host "   ERROR: $_" -ForegroundColor Red
}
Write-Host ""

# Test 2: GPU compute (with gpu.compute capability) - should return 42
Write-Host "2. Testing gpu-compute (with gpu.compute capability)..." -ForegroundColor Yellow
try {
    $jobId = Submit-Job -ModuleId "gpu-compute" -Capabilities @("gpu.compute")
    Write-Host "   Job ID: $jobId" -ForegroundColor Gray
    $result = Get-JobResult -JobId $jobId
    if ($result.status.finished -ne $null) {
        $output = Get-OutputString -Result $result
        Write-Host "   Result: $output (expected: 42)" -ForegroundColor Green
    } else {
        Write-Host "   FAILED: $($result.status.failed)" -ForegroundColor Red
    }
} catch {
    Write-Host "   ERROR: $_" -ForegroundColor Red
}
Write-Host ""

# Test 3: Logging test (with logging capability) - should return 100
Write-Host "3. Testing logging-test (with logging capability)..." -ForegroundColor Yellow
try {
    $jobId = Submit-Job -ModuleId "logging-test" -Capabilities @("logging")
    Write-Host "   Job ID: $jobId" -ForegroundColor Gray
    $result = Get-JobResult -JobId $jobId
    if ($result.status.finished -ne $null) {
        $output = Get-OutputString -Result $result
        Write-Host "   Result: $output (expected: 100)" -ForegroundColor Green
    } else {
        Write-Host "   FAILED: $($result.status.failed)" -ForegroundColor Red
    }
} catch {
    Write-Host "   ERROR: $_" -ForegroundColor Red
}
Write-Host ""

# Test 4: GPU compute WITHOUT capability (should fail)
Write-Host "4. Testing gpu-compute WITHOUT capability (should fail)..." -ForegroundColor Yellow
try {
    $jobId = Submit-Job -ModuleId "gpu-compute" -Capabilities @()
    Write-Host "   Job ID: $jobId" -ForegroundColor Gray
    $result = Get-JobResult -JobId $jobId
    if ($result.status.failed) {
        Write-Host "   Correctly failed (expected behavior)" -ForegroundColor Green
    } else {
        Write-Host "   Unexpected success" -ForegroundColor Red
    }
} catch {
    Write-Host "   ERROR: $_" -ForegroundColor Red
}
Write-Host ""

# Test 5: Ultra simple (no capabilities) - should return 42
Write-Host "5. Testing ultra-simple (no capabilities)..." -ForegroundColor Yellow
try {
    $jobId = Submit-Job -ModuleId "ultra-simple"
    Write-Host "   Job ID: $jobId" -ForegroundColor Gray
    $result = Get-JobResult -JobId $jobId
    if ($result.status.finished -ne $null) {
        $output = Get-OutputString -Result $result
        Write-Host "   Result: $output (expected: 42)" -ForegroundColor Green
    } else {
        Write-Host "   FAILED: $($result.status.failed)" -ForegroundColor Red
    }
} catch {
    Write-Host "   ERROR: $_" -ForegroundColor Red
}
Write-Host ""

Write-Host "=== All tests complete ===" -ForegroundColor Cyan
