use derive_more::{From, Into};

// ---------- NEW TYPE PATTERN ----------

#[derive(Debug, Clone, From, Into)]
struct OrderId(u32);

#[derive(Debug, Clone, From, Into)]
struct Amount(f64);

// ---------- STATES ----------

struct New;
struct Paid;
struct Shipped;

// ---------- ORDER STRUCT ----------

struct Order<State> {
    id: OrderId,
    amount: Amount,
    state: std::marker::PhantomData<State>,
}

// ---------- IMPLEMENTATIONS ----------

// Створення нового замовлення
impl Order<New> {
    fn new(id: OrderId, amount: Amount) -> Self {
        Order {
            id,
            amount,
            state: std::marker::PhantomData,
        }
    }

    fn pay(self) -> Order<Paid> {
        println!("Order paid");
        Order {
            id: self.id,
            amount: self.amount,
            state: std::marker::PhantomData,
        }
    }
}

// Перехід Paid -> Shipped
impl Order<Paid> {
    fn ship(self) -> Order<Shipped> {
        println!("Order shipped");
        Order {
            id: self.id,
            amount: self.amount,
            state: std::marker::PhantomData,
        }
    }
}

// Фінальний стан
impl Order<Shipped> {
    fn finish(self) {
        println!("Order completed!");
    }
}

// ---------- MAIN ----------

fn main() {
    let order = Order::<New>::new(OrderId(1), Amount(100.0));

    let order = order.pay();     // OK
    let order = order.ship();    // OK

    order.finish();

    //  ПОМИЛКА КОМПІЛЯЦІЇ:
    // let order = Order::<New>::new(OrderId(1), Amount(100.0));
    // let order = order.ship(); // НЕ СКОМПІЛЮЄТЬСЯ
}