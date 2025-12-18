{{/*
Expand the name of the chart.
*/}}
{{- define "beejs.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "beejs.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "beejs.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "beejs.labels" -}}
helm.sh/chart: {{ include "beejs.chart" . }}
{{ include "beejs.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "beejs.selectorLabels" -}}
app.kubernetes.io/name: {{ include "beejs.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "beejs.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "beejs.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Beejs specific labels
*/}}
{{- define "beejs.component" -}}
app.kubernetes.io/component: runtime
{{- end }}

{{/*
Performance configuration
*/}}
{{- define "beejs.performance.config" -}}
{{- $config := dict -}}
{{- $_ := set $config "jit" .Values.performance.jitOptimization -}}
{{- $_ := set $config "zeroCopyIO" .Values.performance.zeroCopyIO -}}
{{- $_ := set $config "memoryPool" .Values.performance.memoryPool.enabled -}}
{{- toYaml $config }}
{{- end }}
