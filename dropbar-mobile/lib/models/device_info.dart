class DeviceInfo {
  final String id;
  final String name;
  final String deviceType;
  final String ip;
  final int port;
  DateTime lastSeen;

  DeviceInfo({
    required this.id,
    required this.name,
    required this.deviceType,
    required this.ip,
    required this.port,
    required this.lastSeen,
  });

  factory DeviceInfo.fromJson(Map<String, dynamic> json) {
    return DeviceInfo(
      id: json['id'] as String? ?? '',
      name: json['name'] as String? ?? 'Unknown',
      deviceType: json['device_type'] is Map
          ? (json['device_type'] as Map).keys.first.toString()
          : (json['device_type']?.toString() ?? 'Unknown'),
      ip: json['ip'] as String? ?? '127.0.0.1',
      port: json['port'] as int? ?? 53318,
      lastSeen: json['last_seen'] != null
          ? DateTime.tryParse(json['last_seen'] as String) ?? DateTime.now()
          : DateTime.now(),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'name': name,
      'device_type': deviceType,
      'ip': ip,
      'port': port,
      'last_seen': lastSeen.toIso8601String(),
    };
  }
}

