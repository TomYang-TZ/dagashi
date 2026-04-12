import Foundation

struct PullCollection: Codable {
    var pulls: [PullMeta]
    var unique_characters: [String: Int]?
}

struct PullMeta: Codable {
    var date: String
    var character: String
    var scene: String
    var rarity: String
    var flavor_text: String
    var source: String
    var color_mode: String
    var frame_count: Int
    var anime_title: String
    var anime_rank: Int
    var source_url: String?
    var search_query: String?
}
