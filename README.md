# valo-member-bot

以下は、`tokio::spawn()` を使用して独立させるべき非同期タスクの候補を列挙します。これらは、他の処理と独立して動作する必要がある、または長時間実行される可能性があるタスクです。

---

### **1. 長時間実行されるタスク**
- **例**: 2時間後にリマインダーを送信する処理。
  - 該当箇所: `bot.rs` の `message` メソッド内。
  - 理由: メインの非同期タスクをブロックせず、バックグラウンドで実行する必要があるため。

  ```rust
  tokio::spawn(async move {
      tokio::time::sleep(tokio::time::Duration::from_secs(2 * 60 * 60)).await;
      // リマインダー送信処理
  });
  ```

---

### **2. Redis や外部リソースへの非同期操作**
- **例**: Redis にデータを保存する処理。
  - 該当箇所: `types.rs` の `store_webhook_data` メソッド。
  - 理由: データ保存処理は他の処理と独立しており、非同期タスクとして分離することで効率的に実行できる。

  ```rust
  tokio::spawn(async move {
      let mut conn = conn;
      if let Err(e) = conn.hset_multiple(id.get(), &fields_value).await {
          tracing::error!(error = %e, "Failed to store webhook data");
      }
  });
  ```

---

### **3. ユーザーごとに独立して動作するタスク**
- **例**: 各ユーザーの「サーバー選択」や「モード選択」などの操作。
  - 該当箇所: `bot.rs` の `interaction_create` メソッド内。
  - 理由: 各ユーザーの操作は独立しており、他のユーザーの操作に影響を与えないようにする必要がある。

  ```rust
  tokio::spawn(async move {
      if let Err(e) = self.server(component.user.id, &ctx.http, &component).await {
          tracing::warn!(error = %e, "Failed to create server selection interaction");
      }
  });
  ```

---

### **4. メッセージ送信や編集の処理**
- **例**: Webhook を使用したメッセージ送信や編集。
  - 該当箇所: `panels/send.rs` の `send` メソッドや `panels/edit.rs` の `edit` メソッド。
  - 理由: メッセージ送信や編集は非同期 I/O 操作であり、他の処理と独立して実行するのが適切。

  ```rust
  tokio::spawn(async move {
      if let Err(e) = webhook.execute(http, true, webhook_message).await {
          tracing::warn!(error = %e, "Failed to send webhook message");
      }
  });
  ```

---

### **5. タスクのキャンセルが必要な処理**
- **例**: 期限切れの募集を削除する処理。
  - 該当箇所: `panels.rs` の `handle_expired` メソッド。
  - 理由: 期限切れの募集を削除する処理は、他の処理と独立しており、必要に応じてキャンセル可能にするべき。

  ```rust
  tokio::spawn(async move {
      if let Err(e) = self::delete(http, redis_client, component.message.id).await {
          tracing::warn!(error = %e, "Failed to delete expired panel");
      }
  });
  ```

---

### **6. ボタン操作に関連する処理**
- **例**: 「参加する」「参加をやめる」「削除」ボタンの操作。
  - 該当箇所: `bot.rs` の `interaction_create` メソッド内。
  - 理由: 各ボタン操作は独立した処理であり、非同期タスクとして分離することで効率的に実行できる。

  ```rust
  tokio::spawn(async move {
      match buttons::join(&self.redis_client, component.user.id, component.message.id).await {
          Ok(response) => { /* 処理 */ },
          Err(e) => tracing::warn!(error = %e, "Failed to join"),
      }
  });
  ```

---

### **7. 外部 API 呼び出し**
- **例**: Webhook の作成や取得。
  - 該当箇所: `types.rs` の `get_webhook` メソッド。
  - 理由: 外部 API 呼び出しは非同期 I/O 操作であり、他の処理と独立して実行するのが適切。

  ```rust
  tokio::spawn(async move {
      if let Err(e) = conn.set("webhook_url", &webhook_url).await {
          tracing::warn!(error = %e, "Failed to store webhook URL in Redis");
      }
  });
  ```

---

### **まとめ**

以下のようなタスクは `tokio::spawn()` を使用して独立させるべきです：

1. 長時間実行されるタスク（例: リマインダー送信）。
2. Redis や外部リソースへの非同期操作。
3. ユーザーごとに独立して動作するタスク（例: サーバー選択、モード選択）。
4. メッセージ送信や編集の処理。
5. タスクのキャンセルが必要な処理（例: 期限切れの募集削除）。
6. ボタン操作に関連する処理（例: 参加、削除）。
7. 外部 API 呼び出し。

これらのタスクを `tokio::spawn()` で分離することで、効率的かつスケーラブルな非同期処理を実現できます。ただし、軽量な処理や結果がすぐに必要な処理については、`tokio::spawn()` を避けるべきです。
